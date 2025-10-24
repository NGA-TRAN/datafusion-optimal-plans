#[path = "setup_tables.rs"]
mod setup_tables;
#[path = "test_utils.rs"]
mod test_utils;

use arrow::util::pretty::pretty_format_batches;
use datafusion::error::Result;
use datafusion::physical_plan::displayable;
use datafusion::prelude::{SessionConfig, SessionContext};
use std::error::Error;

// This test ensures probe-side files remain in separate partitions to preserve data locality and sort order during the join.
// It intentionally uses two probe-side files and one build-side file.
#[tokio::test]
async fn test_join_aggregation() -> Result<(), Box<dyn Error>> {
    // Create SessionContext with optimized settings for partitioned data
    let config = SessionConfig::new()
        .with_target_partitions(1) // Keep context as single partition
        .with_repartition_joins(false) // Avoid repartitioning since data is already partitioned correctly
        .with_repartition_aggregations(false) // Avoid repartitioning for aggregations
        .with_repartition_file_scans(false) // Prevent repartitioning of file scans
        .with_collect_statistics(true) // Enable statistics collection for better optimization
        .with_prefer_existing_sort(true); // Prefer existing sort orders to avoid re-sorting

    let ctx = SessionContext::new_with_config(config);

    // Setup tables
    setup_tables::setup_tables(&ctx, "testdata/dimension1/", 1, "testdata/fact2/", 2)
        .await
        .expect("Failed to setup tables");

    // Verify query results
    let join_query = "SELECT  f.f_dkey f_key,
                date_bin(INTERVAL '30 seconds', f.timestamp) time_bin,
                d.service,
                max(f.value) max_bin_val
        FROM  dimension d
        INNER JOIN fact f ON d.d_dkey = f.f_dkey
        WHERE d.env = 'prod'
        GROUP BY f_key, time_bin, service
        ORDER BY f_key, time_bin, service";
    let join_results = ctx.sql(join_query).await?.collect().await?;
    let formatted_results = pretty_format_batches(&join_results)?;
    insta::assert_snapshot!(formatted_results, @r"
    +-------+---------------------+---------+-------------+
    | f_key | time_bin            | service | max_bin_val |
    +-------+---------------------+---------+-------------+
    | B     | 2023-01-01T09:00:00 | log     | 82.4        |
    | B     | 2023-01-01T09:00:30 | log     | 85.6        |
    | B     | 2023-01-01T09:12:30 | log     | 120.0       |
    | B     | 2023-01-01T10:00:00 | log     | 278.9       |
    | B     | 2023-01-01T10:00:30 | log     | 185.6       |
    | B     | 2023-01-01T10:12:30 | log     | 810.0       |
    | C     | 2023-01-01T09:00:00 | log     | 310.2       |
    | C     | 2023-01-01T09:00:30 | log     | 300.0       |
    | C     | 2023-01-01T09:12:30 | log     | 275.4       |
    | C     | 2023-01-01T10:00:00 | log     | 380.2       |
    | C     | 2023-01-01T10:00:30 | log     | 350.0       |
    | C     | 2023-01-01T10:12:30 | log     | 205.4       |
    | D     | 2023-01-01T10:00:00 | trace   | 72.1        |
    +-------+---------------------+---------+-------------+
    ");

    // Display execution plan
    let join_df_explain = ctx.sql(join_query).await?;
    let physical_plan = join_df_explain.create_physical_plan().await?;
    let plan_display = displayable(physical_plan.as_ref()).indent(true).to_string();
    test_utils::insta_settings().bind(|| {
        insta::assert_snapshot!(plan_display, @r#"
        SortExec: expr=[f_key@0 ASC NULLS LAST, time_bin@1 ASC NULLS LAST, service@2 ASC NULLS LAST], preserve_partitioning=[false]
          ProjectionExec: expr=[f_dkey@0 as f_key, date_bin(IntervalMonthDayNano("IntervalMonthDayNano { months: 0, days: 0, nanoseconds: 30000000000 }"),f.timestamp)@1 as time_bin, service@2 as service, max(f.value)@3 as max_bin_val]
            AggregateExec: mode=Final, gby=[f_dkey@0 as f_dkey, date_bin(IntervalMonthDayNano("IntervalMonthDayNano { months: 0, days: 0, nanoseconds: 30000000000 }"),f.timestamp)@1 as date_bin(IntervalMonthDayNano("IntervalMonthDayNano { months: 0, days: 0, nanoseconds: 30000000000 }"),f.timestamp), service@2 as service], aggr=[max(f.value)], ordering_mode=PartiallySorted([0, 1])
              SortPreservingMergeExec: [f_dkey@0 ASC NULLS LAST, date_bin(IntervalMonthDayNano("IntervalMonthDayNano { months: 0, days: 0, nanoseconds: 30000000000 }"),f.timestamp)@1 ASC NULLS LAST]
                AggregateExec: mode=Partial, gby=[f_dkey@1 as f_dkey, date_bin(IntervalMonthDayNano { months: 0, days: 0, nanoseconds: 30000000000 }, timestamp@2) as date_bin(IntervalMonthDayNano("IntervalMonthDayNano { months: 0, days: 0, nanoseconds: 30000000000 }"),f.timestamp), service@0 as service], aggr=[max(f.value)], ordering_mode=PartiallySorted([0, 1])
                  CoalesceBatchesExec: target_batch_size=8192
                    HashJoinExec: mode=CollectLeft, join_type=Inner, on=[(d_dkey@0, f_dkey@0)], projection=[service@1, f_dkey@2, timestamp@3, value@4]
                      CoalesceBatchesExec: target_batch_size=8192
                        FilterExec: env@1 = prod, projection=[d_dkey@0, service@2]
                          DataSourceExec: file_groups={1 group: [[/testdata/dimension1/dimension_1.parquet]]}, projection=[d_dkey, env, service], output_ordering=[env@1 ASC NULLS LAST, service@2 ASC NULLS LAST], file_type=parquet, predicate=env@1 = prod, pruning_predicate=env_null_count@2 != row_count@3 AND env_min@0 <= prod AND prod <= env_max@1, required_guarantees=[env in (prod)]
                      DataSourceExec: file_groups={2 groups: [[/testdata/fact2/fact_1.parquet], [/testdata/fact2/fact_2.parquet]]}, projection=[f_dkey, timestamp, value], output_ordering=[f_dkey@0 ASC NULLS LAST, timestamp@1 ASC NULLS LAST], file_type=parquet, predicate=DynamicFilterPhysicalExpr [ true ]
        "#);
    });

    Ok(())
}

// TODOs:
// 1. Test with multiple build-side files grouped into a single partition (one hash table), since dimension files share similar content.
//   Keep probe-side files in separate partitions to maintain data locality and sort order during the join.
// 2. Test aggregation optimization using statistics and additional config or physical rules to enable single-step execution.
// 3. Test with one more time aggregation in the query
