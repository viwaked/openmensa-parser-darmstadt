# Mensa Darmstadt App GraphQL

- Production instance: `https://mensa.k8s.incloud.de/graphql`
- Needs a `Authorization` header: The app sends a random UUIDv4 that is generated on initial launch (eg. `b4a977c0-4236-4029-bd8f-587c917903ad`), however the server accepts any set value
- Introspection seems disabled
- Apollo GraphQL server

## Canteen ID's

- Mensa Stadtmitte: 1
- Mensa Lichtwiese: 2
- Mensa Schöfferstrasse: 3
- Mensa Dieburg: 4
- Bistro Haardtring: 5
- Schöffers Campusrestaurant: 7

## Request structure
`POST https://mensa.k8s.incloud.de/graphql`
Headers: `Authorization: some-value`, `Content-Type: application/json`
JSON-Body:
```json
{
  "operationName": "LocationsQuery",
  "variables": {},
  "query": "query LocationsQuery { locations { id name coordinates { latitude longitude } type openingHours description image { id url } } }"
}
```

## [Reverse engineered queries and mutations](./reverse_engineered)

- dumped from the android app
