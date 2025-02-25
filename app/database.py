import os

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

_db_user = os.getenv("DATABASE_USER", "blokmap")
_db_pwd = os.getenv("DATABASE_PASSWORD", "appel")
_db_host = os.getenv("DATABASE_HOST", "127.0.0.1")
_db_port = os.getenv("DATABASE_PORT", "5432")
_db_name = os.getenv("DATABASE_NAME", "blokmap")

url = f"postgresql+psycopg://{_db_user}:{_db_pwd}@{_db_host}:{_db_port}/{_db_name}"

engine = create_engine(url)

Session = sessionmaker(bind=engine)
session = Session()
