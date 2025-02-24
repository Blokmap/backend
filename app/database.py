import os

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

_db_user = os.environ.get("DATABASE_USER")
_db_pwd = os.environ.get("DATABASE_PASSWORD")
_db_host = os.environ.get("DATABASE_HOST")
_db_name = os.environ.get("DATABASE_NAME")

url = f"postgresql+psycopg://{_db_user}:{_db_pwd}@{_db_host}:5432/{_db_name}"

engine = create_engine(url)

Session = sessionmaker(bind=engine)
session = Session()
