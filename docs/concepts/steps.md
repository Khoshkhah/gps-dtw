# The steps

## 1. Clean the GPS points

The cleaning:

- removes outliers (a speed of more than 50 m/s between two points);
- smooths the velocity, but only for steps of 2 s or less. After a longer gap, a point keeps its GPS position;
- calculates the speed and the bearing at each point;
- marks the stops.

## 2. Interpolate at 1 s

A cubic Hermite curve (in Bezier form) goes through each GPS point. Its control points are at
`gap time × velocity / 3` from the two end points. The result is one point each second. GPS-DTW keeps these points as
they are: it removes only exact repeats (stopped points) and adds no point.

## 3. Confidence weights

Each point gets a weight from its time to the nearest real GPS point. See [The confidence weight](confidence.md).

## 4. Divide the roads

1. **The search radius (the corridor).** GPS-DTW uses only the edges near the input points. Each point has its own
   radius. At a real GPS point it is `corridor_min` (30 m): the GPS point is near its road. At an interpolated point
   it grows with the seconds to the nearest GPS point, up to `r = max(corridor_min, corridor_gap × median distance
   between two GPS points)` at the middle of a gap. With the defaults, `r` equals the GPS spacing: about 160 m at
   15 s and 270 m at 30 s in Monaco. Between two GPS points the true road can go far from the interpolated curve (a
   winding road), so the radius must grow there. Near the GPS points it does not need to: there the small radius
   keeps out the far edges, and the DTW has fewer road points.
2. **The division.** Each edge gets a point each `speed × 1 s` from its start, along its shape, and its end point.
3. **The connections.** A road point follows the previous point on its edge. The last point of an edge connects to the
   first point of each edge that can follow it: the `turns` of the network (duckOSM's `edge_graph`, without the turns
   that a turn restriction forbids), or, without `turns`, each edge that starts at its end node.

See [One second on both sides](time-division.md).

## 5. DTW and the route

DTW ([What DTW is](dtw.md)) aligns the input with the road graph and finds the alignment with the lowest total cost (see [Segment mode and node mode](cost.md)).
The backtrack gives the aligned road segments. The route is the sequence of their edges, each edge once for each pass.
