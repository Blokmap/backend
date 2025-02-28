import os

JWT_SECRET_KEY = ""

if os.path.isfile("/run/secrets/jwt-secret-key"):
    with open("/run/secrets/jwt-secret-key") as f:
        JWT_SECRET_KEY = f.read().strip()

JWT_ALGORITHM = os.getenv("JWT_ALGORITHM", default="HS256")

ACCESS_TOKEN_EXPIRE_MINUTES = int(
    os.getenv("ACCESS_TOKEN_EXPIRE_MINUTES", default="30")
)
