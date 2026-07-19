## build runner
FROM oven/bun:1-alpine AS build-runner

# Set temp directory
WORKDIR /tmp/app

# Install dependencies
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Move source files
COPY src ./src
COPY tsconfig.json .

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

# Start bot
CMD [ "bun", "dist/main.ts" ]
