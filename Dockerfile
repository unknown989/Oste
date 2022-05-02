FROM node:lts-alpine

WORKDIR /app/

# Preparing
RUN apk update
RUN apk add git wget
RUN git clone https://github.com/unknown989/Oste.git

# Setting up the client
RUN rm -rf /app/Oste/server
RUN mv /app/Oste/client/* /app/Oste/
RUN rm -rf /app/Oste/client/

# Settings up the webserver
RUN npm install -g httpserver