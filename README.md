## promg

A small CLI tool to create a plot from a [Prometheus range query][prom-range-query].

### Usage

```
Generate a plot from a Prometheus range query

Usage: promg [OPTIONS] <QUERY>...

Arguments:
  <QUERY>...  The Prometheus range query

Options:
  -e, --endpoint <ENDPOINT>  The Prometheus server endpoint [default: http://localhost:9090]
  -s, --step <STEP>          The range query step (in seconds) [default: 60]
      --open                 Open the plot in the browser
      --html                 Write the plot as embeddable HTML to stdout
  -h, --help                 Print help information
```

[prom-range-query]: https://prometheus.io/docs/prometheus/latest/querying/api/#range-queries
