# WarRoom collector. Build from the repo root:  docker compose up -d --build
FROM node:24-alpine

WORKDIR /app
# Node runs the TypeScript sources directly; no build step and no npm dependencies.
RUN echo '{"type":"module"}' > package.json
COPY src/api/types.ts src/api/types.ts
COPY src/domain/ src/domain/
COPY collector/ collector/

USER node
CMD ["node", "--max-old-space-size=64", "collector/collect.ts"]
