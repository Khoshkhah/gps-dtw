# Effect of the parameters

The sample of every third route. Segment mode unless stated.

## Search radius

The radius = `max(corridor_min, corridor_gap × median distance between two GPS points)`.

| `corridor_gap` | Radius at 30 s | 15 s | 30 s | Time for each trace |
|---|---|---|---|---|
| 0.5 | about 134 m | 0.928 / 0.949 | 0.832 / 0.870 | 0.04 s |
| **1.0 (default)** | about 270 m | 0.939 / 0.954 | 0.908 / 0.919 | 0.07 s |
| 1.5 | about 400 m | 0.939 / 0.954 | 0.908 / 0.919 | 0.09 s |
| 2.0 | about 540 m | 0.939 / 0.954 | 0.908 / 0.919 | 0.11 s |

With 0.5, the true road of some 30 s traces goes out of the radius on a winding road, and the match fails almost
completely (15 traces cause half of the recall loss). A radius larger than the GPS spacing gives no more gain.

## Bearing weight with the default radius

| Weight | 15 s | 30 s |
|---|---|---|
| 0 | 0.939 / 0.954 | 0.908 / 0.919 |
| 0.05 | 0.940 / 0.954 | 0.895 / 0.922 |
| 0.1 | 0.938 / 0.955 | 0.892 / 0.923 |
| 0.3 | 0.936 / 0.962 | 0.848 / 0.910 |

The tables below were measured with the old radius (`corridor_gap` 0.5).

## Mode

| Mode | 15 s | 30 s |
|---|---|---|
| Segment | 0.928 / 0.949 | 0.832 / 0.870 |
| Node | 0.917 / 0.957 | 0.823 / 0.885 |

## Bearing weight

| Weight | 180° counts as | 15 s | 30 s |
|---|---|---|---|
| 0 | | 0.928 / 0.949 | 0.832 / 0.870 |
| 0.1 | 18 m | 0.926 / 0.949 | 0.824 / 0.875 |
| 0.2 | 36 m | 0.926 / 0.951 | 0.815 / 0.875 |
| 0.5 | 90 m | 0.919 / 0.960 | 0.764 / 0.857 |
| 1.0 | 180 m | 0.919 / 0.970 | 0.762 / 0.862 |
| 2.0 | 360 m | 0.901 / 0.960 | 0.743 / 0.852 |

A high weight helps precision at 15 s and decreases recall at 30 s. In the middle of a 30 s gap, the direction of the
interpolated curve is often wrong.

## Time scale τ (node mode)

| τ | 15 s | 30 s |
|---|---|---|
| 3 s | 0.917 / 0.957 | 0.823 / 0.885 |
| 10 s | 0.887 / 0.948 | 0.719 / 0.849 |

## Speed (node mode)

| Speed | 15 s | 30 s |
|---|---|---|
| 8 m/s | 0.909 / 0.957 | 0.803 / 0.884 |
| 12 m/s | 0.917 / 0.957 | 0.823 / 0.885 |
