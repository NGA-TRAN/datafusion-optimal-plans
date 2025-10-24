Assume you have created tables shown in the file `2_cli_create_tables.md`

```SQL
SELECT * FROM dimension_csv;

+--------+------+---------+------+
| d_dkey | env  | service | host |
+--------+------+---------+------+
| A      | dev  | log     | ma   |
| B      | prod | log     | ma   |
| C      | prod | log     | vim  |
| D      | prod | trace   | vim  |
+--------+------+---------+------+
4 row(s) fetched.
```

### Explain

#### Tree Explain

```SQL
-- Default
-- SET datafusion.explain.format = 'tree';

EXPLAIN SELECT * FROM dimension_csv;
+---------------+-------------------------------+
| plan_type     | plan                          |
+---------------+-------------------------------+
| physical_plan | ┌───────────────────────────┐ |
|               | │       DataSourceExec      │ |
|               | │    --------------------   │ |
|               | │          files: 1         │ |
|               | │        format: csv        │ |
|               | └───────────────────────────┘ |
|               |                               |
+---------------+-------------------------------+
1 row(s) fetched. 
```

#### Indent Explain

```SQL
SET datafusion.explain.format = 'indent';


EXPLAIN SELECT * FROM dimension_csv;
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type     | plan                                                                                                                                                                                                   |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| logical_plan  | TableScan: dimension_csv projection=[d_dkey, env, service, host]                                                                                                                                       |
| physical_plan | DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv]]}, projection=[d_dkey, env, service, host], file_type=csv, has_header=true |
|               |                                                                                                                                                                                                        |
+---------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
2 row(s) fetched. 
```

### Logical Plan vs Physical Plan

#### Logical Plan
- Plan of the SQL
- Same on any machines, any data layout, any statistics
- Not used to execute yet

#### Physical Plan
Generated from logical plan for
- Current machine (e.g number of CPUs)
- Current data layout (number of files, how they are partitioned, sorted, ...)

### Explain Analyze

```SQL
EXPLAIN ANALYZE SELECT * FROM dimension_csv;

+-------------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| plan_type         | plan                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
+-------------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| Plan with Metrics | DataSourceExec: file_groups={1 group: [[Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv]]}, projection=[d_dkey, env, service, host], file_type=csv, has_header=true, metrics=[output_rows=4, elapsed_compute=1ns, batches_split=0, file_open_errors=0, file_scan_errors=0, time_elapsed_opening=239.208µs, time_elapsed_processing=234.958µs, time_elapsed_scanning_total=180.375µs, time_elapsed_scanning_until_data=167.75µs] |
|                   |                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
+-------------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
1 row(s) fetched.
```

### Explain verbose

- To see verbose plans through logical & physical rules
- Mostly for debugging

```SQL
EXPLAIN VERBOSE SELECT * FROM dimension_csv;
```
