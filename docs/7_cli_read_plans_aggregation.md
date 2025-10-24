Assume you have created tables `dimension_parquet` and `dimension_parquet_sorted` shown in the file `2_cli_create_tables.md`

```SQL
SELECT env, count(*) FROM dimension_parquet_sorted GROUP BY env;
+------+----------+
| env  | count(*) |
+------+----------+
| dev  | 1        |
| prod | 3        |
+------+----------+
2 row(s) fetched. 

```

## Understand Aggregation's Group-By Pipeline vs Group-By Hash

```SQL

-- Set data scan in one stream for this example
set datafusion.execution.target_partitions = 1;


-- Data is sorted on the group-by column 
--    --> Group-by Pipeline = keep streaming data up and do aggregation when hitting new key 
EXPLAIN SELECT env, count(*) FROM dimension_parquet_sorted GROUP BY env;
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                               |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: dimension_parquet_sorted.env, count(Int64(1)) AS count(*)                                                                                                                                              |
|               |   Aggregate: groupBy=[[dimension_parquet_sorted.env]], aggr=[[count(Int64(1))]]                                                                                                                                    |
|               |     TableScan: dimension_parquet_sorted projection=[env]                                                                                                                                                           |
| physical_plan | ProjectionExec: expr=[env@0 as env, count(Int64(1))@1 as count(*)]                                                                                                                                                 |
|               |   AggregateExec: mode=Single, gby=[env@0 as env], aggr=[count(Int64(1))], ordering_mode=Sorted     -- ordering_mode=Sorted is the key                                                                              |
|               |     DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[env], output_ordering=[env@0 ASC NULLS LAST], file_type=parquet |
|               |                                                                                                                                                                                                                    |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 

-- Data is NOT sorted on the group-by column
--   --> Group-By Hash: Build hash table for the group-by keys and map values there
EXPLAIN SELECT env, count(*) FROM dimension_parquet GROUP BY env;
+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                       |
+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: dimension_parquet.env, count(Int64(1)) AS count(*)                                                                                                             |
|               |   Aggregate: groupBy=[[dimension_parquet.env]], aggr=[[count(Int64(1))]]                                                                                                   |
|               |     TableScan: dimension_parquet projection=[env]                                                                                                                          |
| physical_plan | ProjectionExec: expr=[env@0 as env, count(Int64(1))@1 as count(*)]                                                                                                         |
|               |   AggregateExec: mode=Single, gby=[env@0 as env], aggr=[count(Int64(1))]                                                                                                   |
|               |     DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[env], file_type=parquet |
|               |                                                                                                                                                                            |
+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 
```


## Three-step GroupBy: Partial Group-By, Repartition,  Final Group-By

```SQL
-- Want to handle data in many partitions/streams and dta is not yet sorted on group-by key
EXPLAIN SELECT env, count(*) FROM dimension_parquet GROUP BY env;
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                               |
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: dimension_parquet.env, count(Int64(1)) AS count(*)                                                                                                                     |
|               |   Aggregate: groupBy=[[dimension_parquet.env]], aggr=[[count(Int64(1))]]                                                                                                           |
|               |     TableScan: dimension_parquet projection=[env]                                                                                                                                  |
| physical_plan | ProjectionExec: expr=[env@0 as env, count(Int64(1))@1 as count(*)]                                                                                                                 |
|               |   AggregateExec: mode=FinalPartitioned, gby=[env@0 as env], aggr=[count(Int64(1))]      -- Step 3: Final Group-By                                                                  |
|               |     CoalesceBatchesExec: target_batch_size=8192                                                                              -- Remember what this does?                           |
|               |       RepartitionExec: partitioning=Hash([env@0], 16), input_partitions=16              -- Step 2: Repartition on the group-by key                                                 |
|               |         -- RepartitionExec: partitioning=RoundRobinBatch(16), input_partitions=1                                             -- Ignore this line for now. Will be explained next   |
|               |           AggregateExec: mode=Partial, gby=[env@0 as env], aggr=[count(Int64(1))]       -- Step 1: Partial Group-By                                                                |
|               |             DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[env], file_type=parquet |
|               |                                                                                                                                                                                    |
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 
```

## When RepartitionExec does not make sense and hurts performance

```SQL
-- Redundant RepartitionExec that hurts performance
EXPLAIN SELECT env, count(*) FROM dimension_parquet GROUP BY env;
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                               |
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: dimension_parquet.env, count(Int64(1)) AS count(*)                                                                                                                     |
|               |   Aggregate: groupBy=[[dimension_parquet.env]], aggr=[[count(Int64(1))]]                                                                                                           |
|               |     TableScan: dimension_parquet projection=[env]                                                                                                                                  |
| physical_plan | ProjectionExec: expr=[env@0 as env, count(Int64(1))@1 as count(*)]                                                                                                                 |
|               |   AggregateExec: mode=FinalPartitioned, gby=[env@0 as env], aggr=[count(Int64(1))]                                                                                                 |
|               |     CoalesceBatchesExec: target_batch_size=8192                                                                                                                                    |
|               |       RepartitionExec: partitioning=Hash([env@0], 16), input_partitions=16                                                                                                         |
|               |         RepartitionExec: partitioning=RoundRobinBatch(16), input_partitions=1      -- This is NOT needed and HURTS performance                                                     |
|               |           AggregateExec: mode=Partial, gby=[env@0 as env], aggr=[count(Int64(1))]                                                                                                  |
|               |             DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[env], file_type=parquet |
|               |                                                                                                                                                                                    |
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+

-- Ticket: todo
```

## Suboptimal Plan

```SQL
-- Data is already sorted on Group-By key but the plan does not take advantage of it

EXPLAIN SELECT env, count(*) FROM dimension_parquet_sorted GROUP BY env;
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                                         |
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: dimension_parquet_sorted.env, count(Int64(1)) AS count(*)                                                                                                                                                        |
|               |   Aggregate: groupBy=[[dimension_parquet_sorted.env]], aggr=[[count(Int64(1))]]                                                                                                                                              |
|               |     TableScan: dimension_parquet_sorted projection=[env]                                                                                                                                                                     |
| physical_plan | ProjectionExec: expr=[env@0 as env, count(Int64(1))@1 as count(*)]                                                                                                                                                           |
|               |   AggregateExec: mode=FinalPartitioned, gby=[env@0 as env], aggr=[count(Int64(1))], ordering_mode=Sorted                                                                                                                     |
|               |     SortExec: expr=[env@0 ASC NULLS LAST], preserve_partitioning=[true]                                                                                                                                                      |
|               |       CoalesceBatchesExec: target_batch_size=8192                                                                                                                                                                            |
|               |         RepartitionExec: partitioning=Hash([env@0], 16), input_partitions=16                                                                                                                                                 |
|               |           RepartitionExec: partitioning=RoundRobinBatch(16), input_partitions=1                                                                                                                                              |
|               |             AggregateExec: mode=Partial, gby=[env@0 as env], aggr=[count(Int64(1))], ordering_mode=Sorted                                                                                                                    |
|               |               DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[env], output_ordering=[env@0 ASC NULLS LAST], file_type=parquet |
|               |                                                                                                                                                                                                                              |
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 
```

### Better Plan 1: Has the knowledge the file is small and use one-step group-by pipeline without setting config param

Something like this

```SQL
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                               |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: dimension_parquet_sorted.env, count(Int64(1)) AS count(*)                                                                                                                                              |
|               |   Aggregate: groupBy=[[dimension_parquet_sorted.env]], aggr=[[count(Int64(1))]]                                                                                                                                    |
|               |     TableScan: dimension_parquet_sorted projection=[env]                                                                                                                                                           |
| physical_plan | ProjectionExec: expr=[env@0 as env, count(Int64(1))@1 as count(*)]                                                                                                                                                 |
|               |   AggregateExec: mode=Single, gby=[env@0 as env], aggr=[count(Int64(1))], ordering_mode=Sorted     -- ordering_mode=Sorted is the key                                                                              |
|               |     DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[env], output_ordering=[env@0 ASC NULLS LAST], file_type=parquet |
|               |                                                                                                                                                                                                                    |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
```

### Better Plan 2: Have one-step group-by on many sorted streams/partitions

Something like this (TODO: SEE IF CAN PUT TOGETHER THIS PLAN WITH SETTING CHANGES or MINOR CODE WORK)

```SQL
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                                         |
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: dimension_parquet_sorted.env, count(Int64(1)) AS count(*)                                                                                                                                                        |
|               |   Aggregate: groupBy=[[dimension_parquet_sorted.env]], aggr=[[count(Int64(1))]]                                                                                                                                              |
|               |     TableScan: dimension_parquet_sorted projection=[env]                                                                                                                                                                     |
| physical_plan | SortPreservingMergeExec: [env@1 ASC NULLS LAST]                                                                                                                                                                              |
|               |   ProjectionExec: expr=[env@0 as env, count(Int64(1))@1 as count(*)]                                                                                                                                                         |          
|               |      AggregateExec: mode=Single, gby=[env@0 as env], aggr=[count(Int64(1))], ordering_mode=Sorted                                                                                                                            |
|               |         CoalesceBatchesExec: target_batch_size=8192                                                                                                                                                                          |
|               |            RepartitionExec: partitioning=RoundRobinBatch(16), input_partitions=1                                                                                                                                             |
|               |               DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[env], output_ordering=[env@0 ASC NULLS LAST], file_type=parquet |
|               |                                                                                                                                                                                                                              |
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
```
