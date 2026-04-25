# graphexa

WASM graph layout algorithms for React Flow graphs.

## Build

```sh
npm run build
```

This runs:

```sh
wasm-pack build --target web --out-dir pkg
```

The generated package in `pkg/` can be imported by the Electron app in the parent
directory:

```sh
cd ..
npm install ./graphexa/pkg
```

Then import it from renderer code:

```ts
import init, { layout_react_flow_graph } from "graphexa";

await init();

const layouted = JSON.parse(
  layout_react_flow_graph(
    JSON.stringify({ nodes, edges }),
    JSON.stringify({
      algorithm: "layered",
      direction: "DOWN",
      spacingX: 120,
      spacingY: 96,
    })
  )
);
```

## API

`layout_react_flow(nodesJson, edgesJson, optionsJson?)`

Returns a JSON array of React Flow nodes. Each node object is preserved and only
its `position` is replaced.

`layout_react_flow_graph(graphJson, optionsJson?)`

Accepts `{ nodes, edges }` and returns the same graph shape with layouted nodes
and unchanged edges.

Supported algorithms:

- `layered` for directed top-down or left-right graphs
- `grid` for compact deterministic placement
- `bfs` for breadth-first multipartite placement from a start node
- `circular` for NetworkX-style circular placement
- `radial` for hub-oriented circular placement
- `force` for a deterministic force-directed pass
- `forceatlas2` for a ForceAtlas2-inspired force-directed pass
- `kamada_kawai` for a weighted shortest-path cost-function layout

React Flow edges can include `weight` or `data.weight`; weighted algorithms use
that numeric value and default missing weights to `1`.

Options use camelCase JSON keys:

```ts
type LayoutOptions = {
  algorithm?:
    | "layered"
    | "grid"
    | "bfs"
    | "circular"
    | "radial"
    | "force"
    | "forceatlas2"
    | "kamada_kawai";
  direction?: "DOWN" | "RIGHT" | "LR"; // layered
  spacingX?: number;
  spacingY?: number;
  nodeWidth?: number;
  nodeHeight?: number;
  centerX?: number;
  centerY?: number;
  start?: string; // bfs
  align?: "vertical" | "horizontal"; // bfs
  scale?: number; // bfs, circular, kamada_kawai
  iterations?: number; // force, kamada_kawai
  maxIter?: number; // forceatlas2
  jitterTolerance?: number; // forceatlas2
  scalingRatio?: number; // forceatlas2
  gravity?: number; // forceatlas2
  distributedAction?: boolean; // forceatlas2
  strongGravity?: boolean; // forceatlas2
  linlog?: boolean; // forceatlas2
  seed?: number; // forceatlas2 initial positions
};
```
