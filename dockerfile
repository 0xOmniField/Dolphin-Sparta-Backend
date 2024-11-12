# Use the official Rust image from the Docker Hub

from registry.cn-hongkong.aliyuncs.com/omni-new/zk-new-base:1.0.0

# Set the working directory inside the container
WORKDIR /usr/src/zkwasm-app

COPY . .

WORKDIR /usr/src/zkwasm-app/ts

RUN npm install

WORKDIR /usr/src/zkwasm-app

RUN make

WORKDIR /usr/src

RUN cp zkwasm-app/deploy/start.sh ./start.sh

EXPOSE 3000

# Run the application
CMD ["sh", "start.sh"]



