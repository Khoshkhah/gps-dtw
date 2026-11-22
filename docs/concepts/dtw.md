# What DTW is

Dynamic time warping (DTW) compares two sequences that have the same shape but different speeds. It pairs each item of
one sequence with one or more items of the other sequence, in order, so that the total cost of the pairs is the lowest
possible.

## Classic DTW: two sequences

Sequence `A` has the items `a₀ … aₙ₋₁`. Sequence `B` has the items `b₀ … bₘ₋₁`. `c(i, j)` is the cost to pair `aᵢ`
with `bⱼ`, for example the distance between them.

DTW fills a table `D` with `n` rows and `m` columns. `D(i, j)` is the lowest total cost of an alignment that ends with
the pair `(aᵢ, bⱼ)`:

```
D(0, 0) = c(0, 0)
D(i, j) = c(i, j) + min( D(i-1, j-1),     both sequences go to the next item
                         D(i-1, j),       only A goes on (bⱼ pairs with more than one item of A)
                         D(i, j-1) )      only B goes on (aᵢ pairs with more than one item of B)
```

The result is `D(n-1, m-1)`. To find the pairs, go back from the last cell and follow the move that gave each minimum.

### A small example

`A` = 1, 2, 3 and `B` = 1, 1, 2, 3. The cost is `|a - b|`.

| D | b = 1 | b = 1 | b = 2 | b = 3 |
|---|---|---|---|---|
| **a = 1** | **0** | **0** | 1 | 3 |
| **a = 2** | 1 | 1 | **0** | 1 |
| **a = 3** | 3 | 3 | 1 | **0** |

The total cost is 0. The first `1` of `A` pairs with the two `1`s of `B`: DTW "warps" the time of `A`. A comparison
point by point (without warping) gives a cost larger than 0, because the sequences have different lengths.

## Why DTW fits map matching

A GPS trace and the road that the vehicle drove have the same shape. They have different speeds: the vehicle goes fast
on one part and slowly on another part, and the GPS has an error. DTW pairs the trace with the road and permits these
speed differences.

## GPS-DTW: a sequence and a graph

In map matching, the road is not one sequence. It is a network with junctions, and we do not know the route before the
match. GPS-DTW changes classic DTW in three ways.

### 1. The columns are road points of a graph

`A` is the input: the GPS trace at one point each second. The columns are all road points in the corridor (see
[One second on both sides](time-division.md)). A road point `v` has predecessors: the previous point on its edge, or,
for the first point of an edge, the last point of each edge that ends at its start node. "The previous column" `j-1`
becomes "a predecessor `u` of `v`":

```
D(i, v) = c(i, v) + min( D(i-1, u) for each predecessor u of v,     both on
                         D(i-1, v),                                 input only
                         D(i, u)   for each predecessor u of v )    road only
```

The road-only moves stay in one row, and the road graph can have loops. So GPS-DTW calculates them with a shortest-path
search (Dijkstra) along the row, not with one step.

### 2. Open start and open end

The vehicle can start and stop at any place on the road. So:

- **open start:** `D(0, v) = c(0, v)` for each road point `v`;
- **open end:** the result is the lowest `D(n-1, v)` of all road points `v`.

### 3. A weight on each input point

The cost has the [confidence weight](confidence.md) `w` of the input point: `c(i, v) = wᵢ × distance`. An interpolated
point far from a GPS point pulls the alignment less. In segment mode, the rows are input segments and the columns are
road segments (see [The cost](cost.md)).

## From the alignment to the route

The backtrack from the best last cell gives one road point (or road segment) for each step. Each road point is on one
edge. The route is the sequence of these edges, with each edge one time for each pass.

## Size

The table has `n × V` cells: `n` input points and `V` road points in the corridor. A 10-minute trace has 600 input
points; the corridor in Monaco has some thousands of road points. GPS-DTW keeps the whole table in memory. A version
with windows and the top `k` road points of each window is a later step.
