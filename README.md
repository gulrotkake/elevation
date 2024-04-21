# Elevation

Generate an elevation profile from a GeoJSON file and a digital elevation model.

## Options
```
  -f, --filename <dem file>
  -g, --geojson <geojson file>
  -s, --steps <STEPS>        Decimate by stepping every STEPS in output SVG
```

## Usage

```
$ elevation --filename dem.tif --geojson route.json > test.svg
```

### Result

![batteriet](https://github.com/gulrotkake/elevation/assets/539077/4bee073c-4db1-4db9-a659-7e3fa68e0e11)
