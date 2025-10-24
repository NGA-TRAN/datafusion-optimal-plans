#[path = "test_utils.rs"]
mod test_utils;

use arrow::util::pretty::pretty_format_batches;
use datafusion::datasource::listing::{
    ListingOptions, ListingTable, ListingTableConfig, ListingTableUrl,
};
use datafusion::error::Result;
use datafusion::physical_plan::displayable;
use datafusion::prelude::{col, SessionContext};
use std::error::Error;
use std::sync::Arc;

/// Sets up external Parquet tables for parallel execution optimization tests
///
/// This creates two tables:
/// - `dimension`: Contains dimension information with columns (d_dkey, env, service, host)
///                WITH ORDER (env, service, host)
/// - `fact`: Contains time series data with columns (f_dkey, timestamp, value)  
///           WITH ORDER (f_dkey, timestamp) and partitioned by timestamp
///
/// # Arguments
/// * `ctx` - The SessionContext to register tables in
/// * `dimension_table_path` - Path to the dimension table directory
/// * `dimension_target_partitions` - Number of target partitions for dimension table
/// * `fact_table_path` - Path to the fact table directory
/// * `fact_target_partitions` - Number of target partitions for fact table
pub async fn setup_tables(
    ctx: &SessionContext,
    dimension_table_path: &str,
    dimension_target_partitions: usize,
    fact_table_path: &str,
    fact_target_partitions: usize,
) -> Result<(), Box<dyn Error>> {
    use datafusion::datasource::file_format::parquet::ParquetFormat;

    // ------------------------------------------------------------
    // Register dimension table from the dimension directory (contains a parquet file)
    let parquet_format_dimension = Arc::new(ParquetFormat::default());
    let dimension_listing_options = ListingOptions::new(parquet_format_dimension)
        .with_file_extension(".parquet")
        .with_target_partitions(dimension_target_partitions)
        .with_collect_stat(true) // Enable statistics collection
        .with_file_sort_order(vec![vec![
            col("env").sort(true, false),     // ASC, nulls last
            col("service").sort(true, false), // ASC, nulls last
            col("host").sort(true, false),    // ASC, nulls last
        ]]);

    let dimension_url = ListingTableUrl::parse(dimension_table_path)?;
    let state = ctx.state();
    let dimension_schema = dimension_listing_options
        .infer_schema(&state, &dimension_url)
        .await?;

    let dimension_config = ListingTableConfig::new(dimension_url)
        .with_listing_options(dimension_listing_options)
        .with_schema(dimension_schema);

    let dimension_table = Arc::new(ListingTable::try_new(dimension_config)?);
    ctx.register_table("dimension", dimension_table)?;

    // ------------------------------------------------------------
    // Register fact table with one Parquet file per partition
    let parquet_format = Arc::new(ParquetFormat::default());
    let listing_options = ListingOptions::new(parquet_format)
        .with_file_extension(".parquet")
        .with_target_partitions(fact_target_partitions)
        .with_collect_stat(true) // Enable statistics collection
        .with_file_sort_order(vec![vec![
            col("f_dkey").sort(true, false),    // ASC, nulls last
            col("timestamp").sort(true, false), // ASC, nulls last
        ]]);

    // Create the table path pointing to the fact directory (contains parquet files)
    let table_url = ListingTableUrl::parse(fact_table_path)?;

    // Infer schema from the parquet files
    let schema = listing_options.infer_schema(&state, &table_url).await?;

    // Create ListingTable configuration
    let config = ListingTableConfig::new(table_url)
        .with_listing_options(listing_options)
        .with_schema(schema);

    // Create and register the ListingTable - this will treat each Parquet file as a partition
    let fact_table = Arc::new(ListingTable::try_new(config)?);
    ctx.register_table("fact", fact_table)?;

    Ok(())
}

#[tokio::test]
async fn test_dimension_table() -> Result<(), Box<dyn Error>> {
    let ctx = SessionContext::new();

    // Setup tables
    setup_tables(&ctx, "testdata/dimension1/", 1, "testdata/fact2/", 2)
        .await
        .expect("Failed to setup tables");

    // Verify query results
    let dimension_query = "SELECT * FROM dimension ORDER BY env, service, host";
    let dimension_results = ctx.sql(dimension_query).await?.collect().await?;
    let formatted_results = pretty_format_batches(&dimension_results)?;
    insta::assert_snapshot!(formatted_results, @r"
    +--------+------+---------+------+
    | d_dkey | env  | service | host |
    +--------+------+---------+------+
    | A      | dev  | log     | ma   |
    | B      | prod | log     | ma   |
    | C      | prod | log     | vim  |
    | D      | prod | trace   | vim  |
    +--------+------+---------+------+
    ");

    // Display execution plan
    let dimension_df_explain = ctx.sql(dimension_query).await?;
    let physical_plan = dimension_df_explain.create_physical_plan().await?;
    let plan_display = displayable(physical_plan.as_ref()).indent(true).to_string();
    test_utils::insta_settings().bind(|| {
        insta::assert_snapshot!(plan_display, @"DataSourceExec: file_groups={1 group: [[/testdata/dimension1/dimension_1.parquet]]}, projection=[d_dkey, env, service, host], output_ordering=[env@1 ASC NULLS LAST, service@2 ASC NULLS LAST, host@3 ASC NULLS LAST], file_type=parquet");
    });

    Ok(())
}

#[tokio::test]
async fn test_fact_table() -> Result<(), Box<dyn Error>> {
    let ctx = SessionContext::new();

    // Setup tables
    setup_tables(&ctx, "testdata/dimension1/", 1, "testdata/fact2/", 2)
        .await
        .expect("Failed to setup tables");

    // Verify query results
    let fact_query = "SELECT * FROM fact ORDER BY f_dkey, timestamp";
    let fact_results = ctx.sql(fact_query).await?.collect().await?;
    let formatted_results = pretty_format_batches(&fact_results)?;
    insta::assert_snapshot!(formatted_results, @r"
    +--------+---------------------+-------+
    | f_dkey | timestamp           | value |
    +--------+---------------------+-------+
    | A      | 2023-01-01T09:00:00 | 95.5  |
    | A      | 2023-01-01T09:00:10 | 102.3 |
    | A      | 2023-01-01T09:00:20 | 98.7  |
    | A      | 2023-01-01T09:12:20 | 105.1 |
    | A      | 2023-01-01T09:12:30 | 100.0 |
    | A      | 2023-01-01T09:12:40 | 150.0 |
    | A      | 2023-01-01T09:12:50 | 120.8 |
    | A      | 2023-01-01T10:00:00 | 18.5  |
    | A      | 2023-01-01T10:00:10 | 35.3  |
    | A      | 2023-01-01T10:00:20 | 55.7  |
    | A      | 2023-01-01T10:12:20 | 100.1 |
    | A      | 2023-01-01T10:12:30 | 44.0  |
    | A      | 2023-01-01T10:12:40 | 350.0 |
    | A      | 2023-01-01T10:12:50 | 320.8 |
    | B      | 2023-01-01T09:00:00 | 75.2  |
    | B      | 2023-01-01T09:00:10 | 82.4  |
    | B      | 2023-01-01T09:00:20 | 78.9  |
    | B      | 2023-01-01T09:00:30 | 85.6  |
    | B      | 2023-01-01T09:12:30 | 80.0  |
    | B      | 2023-01-01T09:12:40 | 120.0 |
    | B      | 2023-01-01T09:12:50 | 92.3  |
    | B      | 2023-01-01T10:00:00 | 175.2 |
    | B      | 2023-01-01T10:00:10 | 182.4 |
    | B      | 2023-01-01T10:00:20 | 278.9 |
    | B      | 2023-01-01T10:00:30 | 185.6 |
    | B      | 2023-01-01T10:12:30 | 810.0 |
    | B      | 2023-01-01T10:12:40 | 720.0 |
    | B      | 2023-01-01T10:12:50 | 222.3 |
    | C      | 2023-01-01T09:00:00 | 300.5 |
    | C      | 2023-01-01T09:00:10 | 285.7 |
    | C      | 2023-01-01T09:00:20 | 310.2 |
    | C      | 2023-01-01T09:00:30 | 295.8 |
    | C      | 2023-01-01T09:00:40 | 300.0 |
    | C      | 2023-01-01T09:12:40 | 250.0 |
    | C      | 2023-01-01T09:12:50 | 275.4 |
    | C      | 2023-01-01T10:00:00 | 310.5 |
    | C      | 2023-01-01T10:00:10 | 225.7 |
    | C      | 2023-01-01T10:00:20 | 380.2 |
    | C      | 2023-01-01T10:00:30 | 205.8 |
    | C      | 2023-01-01T10:00:40 | 350.0 |
    | C      | 2023-01-01T10:12:40 | 200.0 |
    | C      | 2023-01-01T10:12:50 | 205.4 |
    | D      | 2023-01-01T10:00:00 | 24.8  |
    | D      | 2023-01-01T10:00:10 | 72.1  |
    | D      | 2023-01-01T10:00:20 | 42.5  |
    +--------+---------------------+-------+
    ");

    // Display execution plan
    let fact_df_explain = ctx.sql(fact_query).await?;
    let physical_plan = fact_df_explain.create_physical_plan().await?;
    let plan_display = displayable(physical_plan.as_ref()).indent(true).to_string();
    test_utils::insta_settings().bind(|| {
        insta::assert_snapshot!(plan_display, @r"
        SortPreservingMergeExec: [f_dkey@0 ASC NULLS LAST, timestamp@1 ASC NULLS LAST]
          DataSourceExec: file_groups={2 groups: [[/testdata/fact2/fact_1.parquet], [/testdata/fact2/fact_2.parquet]]}, projection=[f_dkey, timestamp, value], output_ordering=[f_dkey@0 ASC NULLS LAST, timestamp@1 ASC NULLS LAST], file_type=parquet
        ");
    });

    Ok(())
}
