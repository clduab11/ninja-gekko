# Build stage
FROM node:20-bookworm AS builder
WORKDIR /app
COPY package*.json ./
# Install dependencies using the lockfile
RUN npm install

COPY . .
# Ensure all types are available
RUN npm run build

# Runtime stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
