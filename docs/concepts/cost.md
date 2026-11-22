# Segment mode and node mode

GPS-DTW can compare the input with the road in two ways. The DTW (the table, the moves, the open ends) is the same in
the two modes. Only the items in the rows and the columns, and the cost of a cell, are different.

| | Node mode | Segment mode (default) |
|---|---|---|
| A row (input) | one input point, at one second | one input segment: from point `i` to point `i+1` (one second) |
| A column (road) | one road point | one road segment: two consecutive road points on one edge (`speed × 1 s`) |
| The cost of a cell | `w × distance between the two points` | `w × (distance between the two middles + bearing_weight × angle difference)` |
| Direction in the cost | no | yes, when `bearing_weight > 0` |
| Parameter | `segment=False` | `segment=True` |

<figure markdown>
<svg viewBox="0 0 640 230" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Node mode pairs points; segment mode pairs segments by their middles" style="max-width:100%;height:auto;font:13px sans-serif">
  <text x="160" y="20" text-anchor="middle" fill="currentColor" font-weight="bold">Node mode</text>
  <text x="480" y="20" text-anchor="middle" fill="currentColor" font-weight="bold">Segment mode</text>
  <line x1="320" y1="30" x2="320" y2="215" stroke="currentColor" stroke-opacity="0.25"/>
  <!-- node mode -->
  <polyline points="20,160 300,160" stroke="#2ec4b6" stroke-width="6" stroke-linecap="round"/>
  <g fill="#2ec4b6"><circle cx="40" cy="160" r="6"/><circle cx="110" cy="160" r="6"/><circle cx="180" cy="160" r="6"/><circle cx="250" cy="160" r="6"/></g>
  <g fill="#fca311"><circle cx="45" cy="95" r="6"/><circle cx="115" cy="85" r="6"/><circle cx="185" cy="100" r="6"/><circle cx="255" cy="90" r="6"/></g>
  <g stroke="currentColor" stroke-dasharray="4 3"><line x1="45" y1="95" x2="40" y2="160"/><line x1="115" y1="85" x2="110" y2="160"/><line x1="185" y1="100" x2="180" y2="160"/><line x1="255" y1="90" x2="250" y2="160"/></g>
  <text x="160" y="200" text-anchor="middle" fill="currentColor">point against point</text>
  <!-- segment mode -->
  <polyline points="345,90 410,80 475,95 540,85 605,95" fill="none" stroke="#fca311" stroke-width="3"/>
  <g fill="#fca311"><circle cx="345" cy="90" r="5"/><circle cx="410" cy="80" r="5"/><circle cx="475" cy="95" r="5"/><circle cx="540" cy="85" r="5"/><circle cx="605" cy="95" r="5"/></g>
  <polyline points="340,160 620,160" stroke="#2ec4b6" stroke-width="6" stroke-linecap="round"/>
  <g fill="#2ec4b6"><circle cx="345" cy="160" r="5"/><circle cx="410" cy="160" r="5"/><circle cx="475" cy="160" r="5"/><circle cx="540" cy="160" r="5"/><circle cx="605" cy="160" r="5"/></g>
  <g fill="currentColor"><rect x="373" y="81" width="8" height="8"/><rect x="438" y="83" width="8" height="8"/><rect x="503" y="86" width="8" height="8"/><rect x="568" y="86" width="8" height="8"/>
  <rect x="373" y="156" width="8" height="8"/><rect x="438" y="156" width="8" height="8"/><rect x="503" y="156" width="8" height="8"/><rect x="568" y="156" width="8" height="8"/></g>
  <g stroke="currentColor" stroke-dasharray="4 3"><line x1="377" y1="89" x2="377" y2="156"/><line x1="442" y1="91" x2="442" y2="156"/><line x1="507" y1="94" x2="507" y2="156"/><line x1="572" y1="94" x2="572" y2="156"/></g>
  <text x="480" y="200" text-anchor="middle" fill="currentColor">segment against segment (middle to middle)</text>
</svg>
<figcaption>Amber: the input (one point each second). Teal: the road (one point each <code>speed × 1 s</code>). Dashed: the distance in the cost. Squares: the middles of the segments.</figcaption>
</figure>

## Node mode

**The rows** are the input points `p₀ … pₙ₋₁` (one each second). **The columns** are the road points (see
[One second on both sides](time-division.md)).

**The cost** of the cell (input point `i`, road point `v`):

```
c(i, v) = wᵢ × |pᵢ − v|
```

- `|pᵢ − v|` = the straight distance in metres.
- `wᵢ` = the [confidence weight](confidence.md) of the input point.

**The connections between columns:** the road point before `v` on its edge, or, for the first point of an edge, the last
point of each edge that ends at its start node.

Node mode uses only positions. It does not know the direction of the vehicle. On a two-way road, the two edges (one in
each direction) have the same points, so only the connections decide the direction.

## Segment mode

**The rows** are the input segments `sᵢ = (pᵢ, pᵢ₊₁)`: `n − 1` segments of one second each. **The columns** are the road
segments `r = (a, b)`: two consecutive road points on one edge. A road segment of zero length is left out.

**The cost** of the cell (input segment `i`, road segment `r`):

```
c(i, r) = w_seg × ( |m(sᵢ) − m(r)| + bearing_weight × angle(sᵢ, r) )
w_seg   = (wᵢ + wᵢ₊₁) / 2
```

- `m(·)` = the middle of a segment: the mean of its two end points.
- `|m(sᵢ) − m(r)|` = the distance between the two middles, in metres.
- `angle(sᵢ, r)` = the smaller angle between the bearings of the two segments, 0 to 180 degrees. A bearing is in degrees
  clockwise from north. 0 = the same direction, 90 = across, 180 = the opposite direction.
- `bearing_weight` = metres for each degree (default 0). With 0.1, the opposite direction costs the same as 18 m.
- `w_seg` = the mean confidence weight of the two end points of the input segment.

**The connections between columns:** the road segment before `r` on its edge, or, for the first segment of an edge, the
last segment of each edge that ends at its start node.

### Why compare segments

- **One second against one second.** An input segment covers one second. A road segment covers `speed × 1 s`, which is
  also about one second of driving. So the two items have the same size.
- **The direction is available.** A segment has a bearing, and a point does not. With `bearing_weight > 0`, a road in
  the wrong direction costs more.
- **The middle is a fair point.** The middle of a segment is less sensitive to the error of one point than the point
  itself.

### The bearing of a short segment

A very short input segment (a vehicle that almost stands) has an unreliable bearing. GPS-DTW uses it as it is. In the
middle of a long gap, the bearing of the interpolated curve can also be wrong. For this reason a high `bearing_weight`
decreases the recall at 30 s (see [Effect of the parameters](../results/parameters.md)).

## The moves (both modes)

| Move | Row (input) | Column (road) |
|---|---|---|
| Both on | the next point or segment | a connected next point or segment |
| Input only | the next | the same (for example, the vehicle stops) |
| Road only | the same | a connected next (the road goes on while the input waits) |

Each move pays the cost of the cell that it enters. The road-only moves stay in one row, and the road graph can have
loops. So GPS-DTW calculates them with Dijkstra along the row. The alignment can start and end at any column (open
ends). See [What DTW is](dtw.md).

## From the alignment to the route

- **Node mode:** each aligned road point is on one edge.
- **Segment mode:** each aligned road segment is on one edge.

The route is the sequence of these edges, with each edge one time for each pass.

## Which mode to use

| Mode | 15 s | 30 s |
|---|---|---|
| Segment | 0.928 / 0.949 | 0.832 / 0.870 |
| Node | 0.917 / 0.957 | 0.823 / 0.885 |

Recall / precision on the Monaco sample (every third route). Segment mode has slightly more recall, and node mode has
slightly more precision. Segment mode is the default because it is the only mode that can use the direction.
