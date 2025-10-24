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
|               |         RepartitionExec: partitioning=RoundRobinBatch(16), input_partitions=1                                                                                                      |
|               |           AggregateExec: mode=Partial, gby=[env@0 as env], aggr=[count(Int64(1))]                                                                                                  |
|               |             DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[env], file_type=parquet |
|               |                                                                                                                                                                                    |
+---------------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 
Elapsed 0.004 seconds.

> EXPLAIN SELECT env, count(*) FROM dimension_parquet_sorted GROUP BY env;
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



set datafusion.execution.target_partitions = 1;



> EXPLAIN SELECT env, count(*) FROM dimension_parquet_sorted GROUP BY env;
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                               |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: dimension_parquet_sorted.env, count(Int64(1)) AS count(*)                                                                                                                                              |
|               |   Aggregate: groupBy=[[dimension_parquet_sorted.env]], aggr=[[count(Int64(1))]]                                                                                                                                    |
|               |     TableScan: dimension_parquet_sorted projection=[env]                                                                                                                                                           |
| physical_plan | ProjectionExec: expr=[env@0 as env, count(Int64(1))@1 as count(*)]                                                                                                                                                 |
|               |   AggregateExec: mode=Single, gby=[env@0 as env], aggr=[count(Int64(1))], ordering_mode=Sorted                                                                                                                     |
|               |     DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet]]}, projection=[env], output_ordering=[env@0 ASC NULLS LAST], file_type=parquet |
|               |                                                                                                                                                                                                                    |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 


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






