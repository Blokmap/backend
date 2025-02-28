"""create user table

Revision ID: 5886db6254e5
Revises:
Create Date: 2025-02-26 17:40:29.129307

"""
from typing import Sequence, Union

from alembic import op
import sqlmodel as sa


# revision identifiers, used by Alembic.
revision: str = '5886db6254e5'
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "user",
        sa.Column("id", sa.BigInteger, primary_key=True),
        sa.Column("username", sa.Text, nullable=False),
        sa.Column("email", sa.Text, nullable=False),
        sa.Column("hashed_password", sa.Text, nullable=False),
        sa.UniqueConstraint("email", name="unique_email"),
    )


def downgrade() -> None:
    op.drop_table("user")
