# Voltrus Lit (Experimental)

This is an **experimental Lit-based frontend** for Voltrus, built with [Lit](https://lit.dev/) web components.

## Status: Experimental

The primary Voltrus frontend is the React app in `../client/`. This Lit variant is an alternative UI exploring:

- Web component architecture (framework-agnostic)
- Smaller bundle size potential
- Shadow DOM encapsulation

## When to use

- **Production / active development:** Use `../client/` (React)
- **Experimentation / web component research:** Use this directory

## Development

```bash
npm install
npm run dev     # Dev server on port 5174
npm run build   # Production build
```

## Notes

- Both frontends are served by the Rust backend via `ServeDir` in `src/api/mod.rs`
- The Lit frontend is available at `/lit/` path
- The React frontend is the default at `/`
