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
- `radial` for hub-oriented circular placement
- `force` for a deterministic force-directed pass

Options use camelCase JSON keys:

```ts
type LayoutOptions = {
  algorithm?: "layered" | "grid" | "radial" | "force";
  direction?: "DOWN" | "RIGHT" | "LR";
  spacingX?: number;
  spacingY?: number;
  nodeWidth?: number;
  nodeHeight?: number;
  centerX?: number;
  centerY?: number;
  iterations?: number;
};
```
