docker build -t docuvault-main -f ./docuvault-main/Dockerfile ./
docker build -t docuvault-file -f ./docuvault-file/Dockerfile ./
docker build -t file-reverse-proxy -f ./file-reverse-proxy/Dockerfile ./
docker build -t docuvault-convert -f ./docuvault-convert/Dockerfile ./
