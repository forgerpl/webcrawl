# Web server-based web crawler

This is more of a toy project, so don't expect full-fledged crawler.

# Running
To run, it's best to use included docker image:

```sh
docker build -t webcrawl .
docker run --rm -ti --name webcrawl -p 3000:3000 webcrawl -a 0.0.0.0:3000
```

And then the API should be accessible at `http://localhost:3000` on the host.

# Quickstart

## Schedule a crawl

```sh
curl -i -XPOST \
    -d '{"url": "http://some.host.example.com", "throttle": 100}' \
    http://localhost:3000/api/crawl
```

## List all crawled domains

```sh
curl -i -XGET http://localhost:3000/api/domains
```

## List URLs for a domain

```sh
curl -i -XGET http://localhost:3000/api/results?id=http://some.host.example.com
```

## List URLs count for a domain

```sh
curl -i -XGET http://localhost:3000/api/results/count?id=http://some.host.example.com
```


# API

## Get all crawled domains
`GET /api/domains`

## Schedule a crawl
`POST /api/crawl`

### Payload:

```json
{
    url: "http://example.com",
    throttle: 50,
}
```

#### where:
- `url`: an url to be crawled
- `throttle`: a maximum number of concurrent requests

### Response:

```json
{
    "id": "http://example.com"
}
```

### Additional status codes:
- `400` - if the payload is malformed, or it contains invalid URL
- `409` - if the crawl is already pending

## Get results of the crawl
`GET /api/results?id={id}`

### Response

A json list of retrieved URLs

### Additional status codes:
- `202` - if the crawl is pending and the result is not yet available
- `404` - if the `id` is not present in the results cache

## Get number of results of the crawl
`GET /api/results/count?id={id}`

### Response:

```json
{
    "http://example.com": 123
}
```

### Additional status codes:
- `202` - if the crawl is pending and the result is not yet available
- `404` - if the `id` is not present in the results cache
