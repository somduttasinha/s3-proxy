# S3 Proxy

A simple proxy server written in Rust to proxy requests to an S3 bucket. This
should be used for serving static, S3-hosted files to users.

## Usage

### Docker

A template `.env.template` file is provided in the root directory.
Set up the environment variables in `.env` and run the following command:

````bash
docker compose up -d
