"""initial migration

Revision ID: 635bd7e0bd8c
Revises:
Create Date: 2025-02-24 16:25:30.347855

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


# revision identifiers, used by Alembic.
revision: str = '635bd7e0bd8c'
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "todo",
        sa.Column("id", sa.BigInteger, primary_key=True),
        sa.Column("text", sa.Text),
        sa.Column("done", sa.Boolean, default=False, server_default="false"),
    )


def downgrade() -> None:
    op.drop_table("todo")
