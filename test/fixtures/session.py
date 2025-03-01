import pytest
from alembic import command
from alembic.config import Config
from sqlmodel import Session, create_engine
from sqlmodel.pool import StaticPool


@pytest.fixture(name="session")
def session_fixture():
    engine = create_engine(
        "sqlite://",
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
    )

    alembic_cfg = Config("alembic.ini")
    alembic_cfg.set_main_option("is_testing", "true")
    with engine.begin() as connection:
        alembic_cfg.attributes["connection"] = connection
        command.upgrade(alembic_cfg, "head")

    with Session(engine) as session:
        yield session
