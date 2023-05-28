# Defining a Rust Build environment
FROM rust:latest as rust-build-env
ARG PROJECT_NAME=rust-price-tracker
ARG BUILD_PROFILE=dev

# Updating the apt-get package manager
RUN apt-get update
RUN apt-get upgrade --yes
RUN apt-get install --yes protobuf-compiler

# Creating a new Rust project
RUN echo ${PROJECT_NAME}
RUN cargo new --bin ${PROJECT_NAME}
WORKDIR /${PROJECT_NAME}

# Copying the Cargo.toml file
COPY ./Cargo.toml ./Cargo.toml

# Building the project
RUN cargo build --profile ${BUILD_PROFILE}

# Removing the default main.rs + Copying the source files
RUN rm src/*.rs
COPY ./src ./src
COPY ./build.rs ./build.rs

# Removing the project-specific dependencies
RUN rm ./target/**/deps/$(echo ${PROJECT_NAME} | sed 's/-/_/g')* # Note: sed is used to replace all '-' with '_' in the project name.
RUN rm ./target/**/${PROJECT_NAME}*
RUN cargo build --profile ${BUILD_PROFILE}

# Printing out the debug directory
RUN ls -la ./target/debug

# Building the actual executation environment
FROM ubuntu:20.04 as rust-exec-env
ARG PROJECT_NAME=rust-price-tracker
ARG BUILD_PROFILE=dev
ARG USERNAME=rust_user

# Updating the apt-get package manager
RUN apt-get update
RUN apt-get upgrade --yes

# Installing certificates
RUN apt-get install --yes ca-certificates

# Installing OpenSSL
RUN apt-get install --yes openssl

# Exposing port 8080
EXPOSE 8080

# Copying the built binary from the Rust Build environment
COPY --from=rust-build-env ./${PROJECT_NAME}/target/debug/${PROJECT_NAME} ./usr/bin/${PROJECT_NAME}

# Running container as user
RUN useradd -mU ${USERNAME} # Note: -m creates a home directory, -U creates a group with the same name as the user.
USER ${USERNAME}

RUN ls -la /usr/local/lib

# Running the binary
CMD ["rust-price-tracker"]