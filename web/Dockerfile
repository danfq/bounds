## build runner
FROM oven/bun:1-alpine AS build-runner

# Set temp directory
WORKDIR /tmp/app

# Install dependencies
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Move source files
COPY src ./src
COPY public ./public
COPY astro.config.mjs tsconfig.json ./

# Build project
RUN bun run build

## production runner
FROM oven/bun:1-alpine AS prod-runner

# Set work directory
WORKDIR /app

# Install production dependencies
COPY package.json bun.lock ./
RUN bun install --production --frozen-lockfile

# Move build files
COPY --from=build-runner /tmp/app/dist ./dist
COPY server.ts .

# Serve the static Astro build with Bun
CMD [ "bun", "server.ts" ]
