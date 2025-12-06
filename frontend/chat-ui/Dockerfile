# Build stage
FROM node:20-bookworm AS builder
WORKDIR /app
COPY package.json ./
# Installing dependencies. Note: No lockfile present in this directory, so using npm install.
# If this is part of a pnpm workspace, consider copying root pnpm-lock.yaml.
RUN npm install
COPY . .
RUN npm run build

# Runtime stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
