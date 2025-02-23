# Stage 1: Export the requirements with poetry.
FROM python:3.12 AS builder

RUN pip install poetry-plugin-export

WORKDIR /app

COPY pyproject.toml poetry.lock ./

RUN poetry export --without-hashes --format=requirements.txt > requirements.txt

# Stage 2: Install the requirements and run the application.
FROM python:3.12

WORKDIR /app

COPY --from=builder /app/requirements.txt .

RUN pip install -r requirements.txt --no-cache-dir

COPY . .

CMD ["uvicorn",  "--host", "0.0.0.0", "--port", "8000", "--reload", "app.main:app"]