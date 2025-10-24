## Install and run datafusion-cli

- Install:  `cargo install datafusion-cli`
  Or build it from the repo `https://github.com/apache/datafusion`. Remember to `cd datafusion-cli` and run `cargo build`
- Run:  `datafusion-cli` or `datafusion-cli --maxrows 100` to display max 100 rows per command

## Common CLI commands

```SQL
show tables;

+---------------+--------------------+-------------+------------+
| table_catalog | table_schema       | table_name  | table_type |
+---------------+--------------------+-------------+------------+
| datafusion    | information_schema | tables      | VIEW       |
| datafusion    | information_schema | views       | VIEW       |
| datafusion    | information_schema | columns     | VIEW       |
| datafusion    | information_schema | df_settings | VIEW       |
| datafusion    | information_schema | schemata    | VIEW       |
| datafusion    | information_schema | routines    | VIEW       |
| datafusion    | information_schema | parameters  | VIEW       |
+---------------+--------------------+-------------+------------+
7 row(s) fetched. 
```

```SQL
select * from information_schema.tables;

+---------------+--------------------+-------------+------------+
| table_catalog | table_schema       | table_name  | table_type |
+---------------+--------------------+-------------+------------+
| datafusion    | information_schema | tables      | VIEW       |
| datafusion    | information_schema | views       | VIEW       |
| datafusion    | information_schema | columns     | VIEW       |
| datafusion    | information_schema | df_settings | VIEW       |
| datafusion    | information_schema | schemata    | VIEW       |
| datafusion    | information_schema | routines    | VIEW       |
| datafusion    | information_schema | parameters  | VIEW       |
+---------------+--------------------+-------------+------------+
7 row(s) fetched. 
```

These commands return nothing when there are no user tables
``` SQL
select * from information_schema.views;
select * from information_schema.columns;
select * from information_schema.schemata;
```

## Configuration Parameters

### See all params:

```SQL
select * from information_schema.df_settings;

+-------------------------------------------------------------------------+---------------------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| name                                                                    | value                     | description                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
+-------------------------------------------------------------------------+---------------------------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| datafusion.catalog.create_default_catalog_and_schema                    | false                     | Whether the default catalog and schema should be created automatically.                                                                                                                                                                                                                                                                                                                                                                                                        |
| datafusion.catalog.default_catalog                                      | datafusion                | The default catalog name - this impacts what SQL queries use if not specified
| ... (additional rows truncated for brevity)
```

### See specific params:

```SQL
select * from information_schema.df_settings where name = 'datafusion.execution.batch_size';
+---------------------------------+-------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| name                            | value | description                                                                                                                                                                         |
+---------------------------------+-------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| datafusion.execution.batch_size | 8192  | Default batch size while creating new batches, it's especially useful for buffer-in-memory batches since creating tiny batches would result in too much metadata memory consumption |
+---------------------------------+-------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
```

```SQL
select * from information_schema.df_settings where name = 'datafusion.explain.format';

+---------------------------+-------+-----------------------------------------------------------------------------------------------------------------------+
| name                      | value | description                                                                                                           |
+---------------------------+-------+-----------------------------------------------------------------------------------------------------------------------+
| datafusion.explain.format | tree  | Display format of explain. Default is "indent". When set to "tree", it will print the plan in a tree-rendered format. |
+---------------------------+-------+-----------------------------------------------------------------------------------------------------------------------+
```

```SQL
select * from information_schema.df_settings where name = 'datafusion.execution.target_partitions';

+----------------------------------------+-------+---------------------------------------------------------------------------------------------------------------------------------------------+
| name                                   | value | description                                                                                                                                 |
+----------------------------------------+-------+---------------------------------------------------------------------------------------------------------------------------------------------+
| datafusion.execution.target_partitions | 16    | Number of partitions for query execution. Increasing partitions can increase concurrency. Defaults to the number of CPU cores on the system |
+----------------------------------------+-------+---------------------------------------------------------------------------------------------------------------------------------------------+
```

### Set values for the config params

```SQL
set datafusion.execution.batch_size=4096;
set datafusion.explain.format = 'indent';
set datafusion.execution.target_partitions = 2;

select * from information_schema.df_settings where name = 'datafusion.execution.batch_size' OR name = 'datafusion.explain.format' OR name = 'datafusion.execution.target_partitions';

+----------------------------------------+--------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| name                                   | value  | description                                                                                                                                                                         |
+----------------------------------------+--------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| datafusion.execution.batch_size        | 4096   | Default batch size while creating new batches, it's especially useful for buffer-in-memory batches since creating tiny batches would result in too much metadata memory consumption |
| datafusion.execution.target_partitions | 2      | Number of partitions for query execution. Increasing partitions can increase concurrency. Defaults to the number of CPU cores on the system                                         |
| datafusion.explain.format              | indent | Display format of explain. Default is "indent". When set to "tree", it will print the plan in a tree-rendered format.                                                               |
+----------------------------------------+--------+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
3 row(s) fetched. 
```

## Other commands:

```SQL
select * from information_schema.routines;
select * from information_schema.parameters;
```


## Some tricks to see schema

```SQL
select * from information_schema.df_settings limit 0;

+------+-------+-------------+
| name | value | description |
+------+-------+-------------+
+------+-------+-------------+
0 row(s) fetched.
```

```SQL
select * from information_schema.df_settings limit 1;

+------------------------------------------------------+-------+-------------------------------------------------------------------------+
| name                                                 | value | description                                                             |
+------------------------------------------------------+-------+-------------------------------------------------------------------------+
| datafusion.catalog.create_default_catalog_and_schema | false | Whether the default catalog and schema should be created automatically. |
+------------------------------------------------------+-------+-------------------------------------------------------------------------+
1 row(s) fetched. 
```

## Quit the CLI

```SQL
\q
```
