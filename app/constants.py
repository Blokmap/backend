import os

# JWT settings.
JWT_ALGORITHM = os.getenv("JWT_ALGORITHM", default="HS256")
JWT_SECRET_KEY = ""

if os.path.isfile("/run/secrets/jwt-secret-key"):
    with open("/run/secrets/jwt-secret-key") as f:
        JWT_SECRET_KEY = f.read().strip()

# Access token settings.
ACCESS_TOKEN_NAME = os.getenv("ACCESS_TOKEN_NAME", default="access_token")
ACCESS_TOKEN_EXPIRE_MINUTES = int(
    os.getenv("ACCESS_TOKEN_EXPIRE_MINUTES", default="30")
)
