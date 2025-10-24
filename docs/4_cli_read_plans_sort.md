Assume you have created tables `dimension_csv` and  `dimension_csv_sorted` shown in the file `2_cli_create_tables.md`


### Table is not sorted

```SQL
SELECT * FROM dimension_csv ORDER BY env, service, host;
+--------+------+---------+------+
| d_dkey | env  | service | host |
+--------+------+---------+------+
| A      | dev  | log     | ma   |
| B      | prod | log     | ma   |
| C      | prod | log     | vim  |
| D      | prod | trace   | vim  |
+--------+------+---------+------+
4 row(s) fetched. 

-- Data is not sorted yet --> must be sorted by SortExec

EXPLAIN SELECT * FROM dimension_csv ORDER BY env, service, host;
+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                     |
+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Sort: dimension_csv.env ASC NULLS LAST, dimension_csv.service ASC NULLS LAST, dimension_csv.host ASC NULLS LAST                                                                                          |
|               |   TableScan: dimension_csv projection=[d_dkey, env, service, host]                                                                                                                                       |
| physical_plan | SortExec: expr=[env@1 ASC NULLS LAST, service@2 ASC NULLS LAST, host@3 ASC NULLS LAST], preserve_partitioning=[false]                                                                                    |
|               |   DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv]]}, projection=[d_dkey, env, service, host], file_type=csv, has_header=true |
|               |                                                                                                                                                                                                          |
+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 
```

### Table is sorted on env, service, host

```SQL
SELECT * FROM dimension_csv_sorted ORDER BY env, service, host;
+--------+------+---------+------+
| d_dkey | env  | service | host |
+--------+------+---------+------+
| A      | dev  | log     | ma   |
| B      | prod | log     | ma   |
| C      | prod | log     | vim  |
| D      | prod | trace   | vim  |
+--------+------+---------+------+
4 row(s) fetched.

-- Data is already sorted  --> No need to resort
EXPLAIN SELECT * FROM dimension_csv_sorted ORDER BY env, service, host;
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                                                                                                            |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Sort: dimension_csv_sorted.env ASC NULLS LAST, dimension_csv_sorted.service ASC NULLS LAST, dimension_csv_sorted.host ASC NULLS LAST                                                                                                                                                            |
|               |   TableScan: dimension_csv_sorted projection=[d_dkey, env, service, host]                                                                                                                                                                                                                       |
| physical_plan | DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv]]}, projection=[d_dkey, env, service, host], output_ordering=[env@1 ASC NULLS LAST, service@2 ASC NULLS LAST, host@3 ASC NULLS LAST], file_type=csv, has_header=true |
|               |                                                                                                                                                                                                                                                                                                 |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched.
```

### Good Subset of Sort Order

```SQL
SELECT * FROM dimension_csv_sorted ORDER BY env, service;
+--------+------+---------+------+
| d_dkey | env  | service | host |
+--------+------+---------+------+
| A      | dev  | log     | ma   |
| B      | prod | log     | ma   |
| C      | prod | log     | vim  |
| D      | prod | trace   | vim  |
+--------+------+---------+------+
4 row(s) fetched. 

-- No SortExec
EXPLAIN SELECT * FROM dimension_csv_sorted ORDER BY env, service;
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                                                                                                            |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Sort: dimension_csv_sorted.env ASC NULLS LAST, dimension_csv_sorted.service ASC NULLS LAST                                                                                                                                                                                                      |
|               |   TableScan: dimension_csv_sorted projection=[d_dkey, env, service, host]                                                                                                                                                                                                                       |
| physical_plan | DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv]]}, projection=[d_dkey, env, service, host], output_ordering=[env@1 ASC NULLS LAST, service@2 ASC NULLS LAST, host@3 ASC NULLS LAST], file_type=csv, has_header=true |
|               |                                                                                                                                                                                                                                                                                                 |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+

```

### Bad Subset of Sort Order

```SQL
SELECT * FROM dimension_csv_sorted ORDER BY env, host;
+--------+------+---------+------+
| d_dkey | env  | service | host |
+--------+------+---------+------+
| A      | dev  | log     | ma   |
| B      | prod | log     | ma   |
| C      | prod | log     | vim  |
| D      | prod | trace   | vim  |
+--------+------+---------+------+

-- Must see SortExec
EXPLAIN SELECT * FROM dimension_csv_sorted ORDER BY env, host;
+---------------+---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                                                                                                              |
+---------------+---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Sort: dimension_csv_sorted.env ASC NULLS LAST, dimension_csv_sorted.host ASC NULLS LAST                                                                                                                                                                                                           |
|               |   TableScan: dimension_csv_sorted projection=[d_dkey, env, service, host]                                                                                                                                                                                                                         |
| physical_plan | SortExec: expr=[env@1 ASC NULLS LAST, host@3 ASC NULLS LAST], preserve_partitioning=[false]                                                                                                                                                                                                       |
|               |   DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv]]}, projection=[d_dkey, env, service, host], output_ordering=[env@1 ASC NULLS LAST, service@2 ASC NULLS LAST, host@3 ASC NULLS LAST], file_type=csv, has_header=true |
|               |                                                                                                                                                                                                                                                                                                   |
+---------------+---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
```

### Projection

```SQL
SELECT host, env FROM dimension_csv_sorted ORDER BY env, service;
+------+------+
| host | env  |
+------+------+
| ma   | dev  |
| ma   | prod |
| vim  | prod |
| vim  | prod |
+------+------+
4 row(s) fetched. 

-- Still has knowledge of sort order
EXPLAIN SELECT host, env FROM dimension_csv_sorted ORDER BY env, service;
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                                          |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | Projection: dimension_csv_sorted.host, dimension_csv_sorted.env                                                                                                                                                               |
|               |   Sort: dimension_csv_sorted.env ASC NULLS LAST, dimension_csv_sorted.service ASC NULLS LAST                                                                                                                                  |
|               |     Projection: dimension_csv_sorted.host, dimension_csv_sorted.env, dimension_csv_sorted.service                                                                                                                             |
|               |       TableScan: dimension_csv_sorted projection=[env, service, host]                                                                                                                                                         |
| physical_plan | DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv]]}, projection=[host, env], output_ordering=[env@1 ASC NULLS LAST], file_type=csv, has_header=true |
|               |                                                                                                                                                                                                                               |
+---------------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched.
```
