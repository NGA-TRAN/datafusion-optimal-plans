Assume you have created tables `dimension_csv` and  `dimension_csv_sorted` and `dimension_parquet_sorted` shown in the file `2_cli_create_tables.md`


### Filter Data

```SQL
SELECT * FROM dimension_csv_sorted WHERE service = 'log';
+--------+------+---------+------+
| d_dkey | env  | service | host |
+--------+------+---------+------+
| A      | dev  | log     | ma   |
| B      | prod | log     | ma   |
| C      | prod | log     | vim  |
+--------+------+---------+------+
3 row(s) fetched. 
```

### Understand RepartitionExec & CoalesceBatchesExec

```SQL
--  RepartitionExec: partitioning=RoundRobinBatch(16): Split 1 stream/partition into 16 streams/partitions in round robin fashion
--     Still preserve order of the streams 
--  CoalesceBatchesExec: target_batch_size=8192: Collect data of EACH stream to 8192(bytes?) before pushing up

EXPLAIN SELECT * FROM dimension_csv_sorted WHERE service = 'log';
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                                                                                                                  |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Filter: dimension_csv_sorted.service = Utf8("log")                                                                                                                                                                                                                                                    |
|               |   TableScan: dimension_csv_sorted projection=[d_dkey, env, service, host], partial_filters=[dimension_csv_sorted.service = Utf8("log")]                                                                                                                                                               |
| physical_plan | CoalesceBatchesExec: target_batch_size=8192                                                                                                                                                                                                                                                           |
|               |   FilterExec: service@2 = log                                                                                                                                                                                                                                                                         |
|               |     RepartitionExec: partitioning=RoundRobinBatch(16), input_partitions=1                                                                                                                                                                                                                             |
|               |       DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv]]}, projection=[d_dkey, env, service, host], output_ordering=[env@1 ASC NULLS LAST, service@2 ASC NULLS LAST, host@3 ASC NULLS LAST], file_type=csv, has_header=true |
|               |                                                                                                                                                                                                                                                                                                       |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 

```

### Why Repartitioned to 16 of them?

Default - 16 : number of CPUs

```SQL
select * from information_schema.df_settings where name = 'datafusion.execution.target_partitions';
+----------------------------------------+-------+---------------------------------------------------------------------------------------------------------------------------------------------+
| name                                   | value | description                                                                                                                                 |
+----------------------------------------+-------+---------------------------------------------------------------------------------------------------------------------------------------------+
| datafusion.execution.target_partitions | 16    | Number of partitions for query execution. Increasing partitions can increase concurrency. Defaults to the number of CPU cores on the system |
+----------------------------------------+-------+---------------------------------------------------------------------------------------------------------------------------------------------+
```


### Why target_batch_size=8192 

Default setting 

```SQL
select * from information_schema.df_settings where name = 'datafusion.execution.batch_size';
+---------------------------------+-------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| name                            | value | description                                                                                                                                                                         |
+---------------------------------+-------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| datafusion.execution.batch_size | 8192  | Default batch size while creating new batches, it's especially useful for buffer-in-memory batches since creating tiny batches would result in too much metadata memory consumption |
+---------------------------------+-------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
```

### Change config params

```SQL
set datafusion.execution.batch_size=4096;
set datafusion.execution.target_partitions = 4;

EXPLAIN SELECT * FROM dimension_csv_sorted WHERE service = 'log';
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                                                                                                                  |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Filter: dimension_csv_sorted.service = Utf8("log")                                                                                                                                                                                                                                                    |
|               |   TableScan: dimension_csv_sorted projection=[d_dkey, env, service, host], partial_filters=[dimension_csv_sorted.service = Utf8("log")]                                                                                                                                                               |
| physical_plan | CoalesceBatchesExec: target_batch_size=4096                                                                                                                                                                                                                                                           |
|               |   FilterExec: service@2 = log                                                                                                                                                                                                                                                                         |
|               |     RepartitionExec: partitioning=RoundRobinBatch(4), input_partitions=1                                                                                                                                                                                                                              |
|               |       DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv]]}, projection=[d_dkey, env, service, host], output_ordering=[env@1 ASC NULLS LAST, service@2 ASC NULLS LAST, host@3 ASC NULLS LAST], file_type=csv, has_header=true |
|               |                                                                                                                                                                                                                                                                                                       |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 
```


### Filter Push Down

```SQL
-- Parquet has statistics --> BloomFilter-like Pushdown
-- Still need filter for correctness
EXPLAIN SELECT * FROM dimension_parquet_sorted WHERE service = 'log';
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Filter: dimension_parquet_sorted.service = Utf8View("log")                                                                                                                                                                                                                                                                                                                                                                                                               |
|               |   TableScan: dimension_parquet_sorted projection=[d_dkey, env, service, host], partial_filters=[dimension_parquet_sorted.service = Utf8View("log")]                                                                                                                                                                                                                                                                                                                      |
| physical_plan | CoalesceBatchesExec: target_batch_size=4096                                                                                                                                                                                                                                                                                                                                                                                                                              |
|               |   FilterExec: service@2 = log                                                                                                                                                                                                                                                                                                                                                                                                                                            |
|               |     RepartitionExec: partitioning=RoundRobinBatch(4), input_partitions=1                                                                                                                                                                                                                                                                                                                                                                                                 |
|               |       DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[d_dkey, env, service, host], output_ordering=[env@1 ASC NULLS LAST, service@2 ASC NULLS LAST, host@3 ASC NULLS LAST], file_type=parquet, predicate=service@2 = log, pruning_predicate=service_null_count@2 != row_count@3 AND service_min@0 <= log AND log <= service_max@1, required_guarantees=[service in (log)] |
|               |                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 
```