# openmensa-parser-darmstadt

- launch with `openmensa-parser-darmstadt <config file>`

example config:
```json
{
  "bind": "0.0.0.0:3000",
  "deployUrl": "http://localhost:3000",
  "canteens": {
    "1": [ "1", "stadtmitte" ],
    "2": [ "2", "lichtwiese" ],
    "3": [ "3", "schoefferstrasse", "schöfferstraße" ],
    "4": [ "4", "dieburg" ],
    "5": [ "5", "haardtring" ],
    "7": [ "7", "schoeffers", "schöffers" ]
  }
}
```
