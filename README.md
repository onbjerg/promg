## promg

A small CLI tool to create a plot from a [Prometheus range query][prom-range-query].

### Usage

```
Generate a plot from a Prometheus range query

Usage: promg [OPTIONS] <QUERY>...

Arguments:
  <QUERY>...
          The Prometheus range query

Options:
  -e, --endpoint <ENDPOINT>
          The Prometheus server endpoint

          [default: http://localhost:9090]

      --start <START>
          The start of the query range (UNIX timestamp)

          Defaults to 24 hours ago.

          [default: 1671784438]

      --end <END>
          The end of the query range (UNIX timestamp)

          Defaults to now.

          [default: 1671870838]

      --step <STEP>
          The range query step (in seconds)

          [default: 60]

  -t, --title <TITLE>
          The title of the plot

      --open
          Open the plot in the browser

      --html
          Write the plot as embeddable HTML to stdout

  -h, --help
          Print help information (use `-h` for a summary)
```

[prom-range-query]: https://prometheus.io/docs/prometheus/latest/querying/api/#range-queries
