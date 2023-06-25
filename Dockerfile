FROM rust:1.70

## Add the wait script to the image
COPY --from=ghcr.io/ufoscout/docker-compose-wait:latest /wait /wait

WORKDIR /usr/src/backend

COPY . .

EXPOSE 8000

## Launch the wait tool and then your application
CMD /wait && cargo install --path . && backend