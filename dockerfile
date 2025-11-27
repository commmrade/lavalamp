FROM rust:1.90-bookworm

# Установка системных зависимостей для OpenCV и FFmpeg
RUN apt-get update && apt-get install -y \
    pkg-config \
    clang \
    libclang-dev \
    libopencv-dev \
    libavcodec-dev \
    libavdevice-dev \
    libavformat-dev \
    libavutil-dev \
    libswresample-dev \
    libswscale-dev \
    libavfilter-dev \
    cmake \
    build-essential \
    ffmpeg \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Рабочая директория
WORKDIR /app

# Копируем проект
COPY . .

# Собираем бинарник в release
RUN cargo build --release

# Добавляем бинарник в PATH
# ENV PATH="/app/target/release:${PATH}"

# Экспонируем порт веб-сервера
EXPOSE 3000

# Команда запуска
CMD ["/app/target/release/lavalamp"]
