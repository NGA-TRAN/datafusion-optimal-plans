### Start CLI
`datafusion-cli --maxrows 100`


### Create tables from single file

- From CSV file

```SQL
-- Not sorted
CREATE EXTERNAL TABLE  dimension_csv
STORED AS CSV 
LOCATION '/Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv'
OPTIONS ('format.has_header' 'true');


-- Sorted
CREATE EXTERNAL TABLE  dimension_csv_sorted
STORED AS CSV 
WITH ORDER (env, service, host)
LOCATION '/Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv'
OPTIONS ('format.has_header' 'true');
```

- From Parquet file

```SQL
-- Not sorted
CREATE EXTERNAL TABLE  dimension_parquet
STORED AS Parquet 
LOCATION '/Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet';


-- Sorted
CREATE EXTERNAL TABLE  dimension_parquet_sorted
STORED AS Parquet
WITH ORDER (env, service, host)
LOCATION '/Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet';
```

### Explore the tables

```SQL
show tables;

+---------------+--------------------+--------------------------+------------+
| table_catalog | table_schema       | table_name               | table_type |
+---------------+--------------------+--------------------------+------------+
| datafusion    | public             | dimension_parquet        | BASE TABLE |
| datafusion    | public             | dimension_parquet_sorted | BASE TABLE |
| datafusion    | public             | dimension_csv_sorted     | BASE TABLE |
| datafusion    | public             | dimension_csv            | BASE TABLE |
| datafusion    | information_schema | tables                   | VIEW       |
| datafusion    | information_schema | views                    | VIEW       |
| datafusion    | information_schema | columns                  | VIEW       |
| datafusion    | information_schema | df_settings              | VIEW       |
| datafusion    | information_schema | schemata                 | VIEW       |
| datafusion    | information_schema | routines                 | VIEW       |
| datafusion    | information_schema | parameters               | VIEW       |
+---------------+--------------------+--------------------------+------------+
11 row(s) fetched. 
```

```SQL
select * from information_schema.views;

+---------------+--------------+--------------------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| table_catalog | table_schema | table_name               | definition                                                                                                                                                         |
+---------------+--------------+--------------------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| datafusion    | public       | dimension_parquet        | CREATE EXTERNAL TABLE dimension_parquet STORED AS PARQUET LOCATION /Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet         |
| datafusion    | public       | dimension_parquet_sorted | CREATE EXTERNAL TABLE dimension_parquet_sorted STORED AS PARQUET LOCATION /Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.parquet  |
| datafusion    | public       | dimension_csv_sorted     | CREATE EXTERNAL TABLE dimension_csv_sorted STORED AS CSV LOCATION /Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv              |
| datafusion    | public       | dimension_csv            | CREATE EXTERNAL TABLE dimension_csv STORED AS CSV LOCATION /Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/dimension1/dimension_1.csv                     |
+---------------+--------------+--------------------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------+

-- TODO: Create a ticket in upstream datafusion to display `WITH ORDER ...` when applicable
```

```SQL
show columns from dimension_csv;

+---------------+--------------+---------------+-------------+-----------+-------------+
| table_catalog | table_schema | table_name    | column_name | data_type | is_nullable |
+---------------+--------------+---------------+-------------+-----------+-------------+
| datafusion    | public       | dimension_csv | d_dkey      | Utf8      | YES         |
| datafusion    | public       | dimension_csv | env         | Utf8      | YES         |
| datafusion    | public       | dimension_csv | service     | Utf8      | YES         |
| datafusion    | public       | dimension_csv | host        | Utf8      | YES         |
+---------------+--------------+---------------+-------------+-----------+-------------+
4 row(s) fetched. 
```

```SQL
select * from dimension_csv;
+--------+------+---------+------+
| d_dkey | env  | service | host |
+--------+------+---------+------+
| A      | dev  | log     | ma   |
| B      | prod | log     | ma   |
| C      | prod | log     | vim  |
| D      | prod | trace   | vim  |
+--------+------+---------+------+
4 row(s) fetched. 

select * from dimension_parquet;
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

### Create tables for many files in a folder

```SQL
-- fact2 folder has 2 parquet files
CREATE EXTERNAL TABLE  fact_parquet_sorted
STORED AS Parquet
WITH ORDER (f_dkey, timestamp)
LOCATION '/Users/hoabinhnga.tran/datafusion-optimal-plans/testdata/fact2';
```

### Create tables & Insert data

**Note:** cannot specify sort order in this case

```SQL
CREATE TABLE dimension_manual(d_dkey string, env string, service string, host string);

INSERT INTO dimension_manual VALUES
    ('A', 'dev', 'log', 'ma'),
    ('B', 'prod', 'log', 'ma');

select * from dimension_manual;
+--------+------+---------+------+
| d_dkey | env  | service | host |
+--------+------+---------+------+
| A      | dev  | log     | ma   |
| B      | prod | log     | ma   |
+--------+------+---------+------+
2 row(s) fetched. 
```


