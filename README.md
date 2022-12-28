## promg

A small CLI tool to create a plot from a [Prometheus range query][prom-range-query].

### Usage

```
Generate a plot from a Prometheus range query

Usage: promg [OPTIONS] --query <QUERY>

Options:
  -e, --endpoint <ENDPOINT>
          The Prometheus server endpoint

          [default: http://localhost:9090]

      --start <START>
          The start of the query range (UNIX timestamp)

          Defaults to 24 hours ago.

          [default: 1672121951]

      --end <END>
          The end of the query range (UNIX timestamp)

          Defaults to now.

          [default: 1672208351]

      --step <STEP>
          The range query step (in seconds)

          [default: 60]

  -t, --title <TITLE>
          The title of the plot.

          Multiple titles can be specified by passing the flag multiple times. The number of titles must match the number of queries.

      --live
          Open the plot in the browser and live-reload it

      --html
          Write the plot as embeddable HTML to stdout

  -q, --query <QUERY>
          The Prometheus range query.

          Multiple queries can be specified by passing the flag multiple times. The number of titles must match the number of queries.

  -h, --help
          Print help information (use `-h` for a summary)
```

[prom-range-query]: https://prometheus.io/docs/prometheus/latest/querying/api/#range-queries
