import os

from sqlalchemy import create_engine
from sqlalchemy.engine import URL
from sqlalchemy.orm import sessionmaker

_db_user = os.environ.get("DATABASE_USER")
_db_name = os.environ.get("DATABASE_NAME")
_db_host = os.environ.get("DATABASE_HOST")

_db_pwd = ""
with open("/run/secrets/db-password", "r") as f:
    _db_pwd = f.read().strip()

url = URL.create(
    drivername="postgresql+psycopg",
    username=_db_user,
    password=_db_pwd,
    host=_db_host,
    port=5432,
    database=_db_name,
)

engine = create_engine(url)

Session = sessionmaker(bind=engine)
session = Session()
