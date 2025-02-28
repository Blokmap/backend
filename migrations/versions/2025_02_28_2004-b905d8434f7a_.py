"""empty message

Revision ID: b905d8434f7a
Revises: 5886db6254e5
Create Date: 2025-02-28 20:04:07.640141

"""

from typing import Sequence, Union

from alembic import op
from app.models.translation import Language
import sqlmodel as sm


# revision identifiers, used by Alembic.
revision: str = "b905d8434f7a"
down_revision: Union[str, None] = "5886db6254e5"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Create the Translation table.
    op.create_table(
        "translation",
        # Columns.
        sm.Column("id", sm.Integer, primary_key=True),
        sm.Column("language", sm.Enum(Language, name="language_enum")),
        sm.Column("translation_key", sm.UUID),
        sm.Column("translation", sm.String),
        sm.Column("created_at", sm.DateTime, server_default=sm.func.now()),
        sm.Column("updated_at", sm.DateTime, server_default=sm.func.now(), onupdate=sm.func.now()),
        # Constraints.
        sm.UniqueConstraint("language", "translation_key"),
    )


def downgrade() -> None:
    # Drop the Translation table
    op.drop_table("translation")
