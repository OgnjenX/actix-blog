# Use the official Rust image to build the application
FROM rust:1.83-slim AS build-env

# Set working directory inside the builder container
WORKDIR /app

# Copy the source code into the builder container
COPY . .

# Install dependencies and build the application
RUN cargo build --release

# Use a distroless base image to run the application
FROM gcr.io/distroless/cc-debian12

# Set the working directory
WORKDIR /app

# Copy the build artifact from the builder container
COPY --from=build-env /app/target/release/actix-blog /app/actix-blog
# Copy the directories with static content from the build container to the final container
COPY --from=build-env /app/posts /app/posts
COPY --from=build-env /app/static /app/static
COPY --from=build-env /app/templates /app/templates

# Expose the port that the Actix app will use
EXPOSE 8080

# Run the Actix app from /app directory
CMD ["./actix-blog"]
