"""create translation tables

Revision ID: b905d8434f7a
Revises:
Create Date: 2025-02-28 20:04:29.129307

"""

from typing import Sequence, Union

import sqlmodel as sm
from alembic import op

from app.models.translation import LanguageEnum

# revision identifiers, used by Alembic.
revision: str = "b905d8434f7a"
down_revision: Union[str, None] = "5886db6254e5"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

# The language enum is used in the translation table.
language_enum = sm.Enum(LanguageEnum, name="language_enum")


def upgrade() -> None:
    # Create the Translation table.

    op.create_table(
        "translation",
        sm.Column("id", sm.Integer, primary_key=True),
        sm.Column("language", language_enum, nullable=False, index=True),
        sm.Column("translation_key", sm.String, nullable=False),
        sm.Column("translation", sm.String, nullable=False, index=True),
        sm.Column("created_at", sm.DateTime, server_default=sm.func.now()),
        sm.Column(
            "updated_at",
            sm.DateTime,
            server_default=sm.func.now(),
            onupdate=sm.func.now(),
        ),
        sm.UniqueConstraint("language", "translation_key"),
    )


def downgrade() -> None:
    op.drop_table("translation")
    language_enum.drop(op.get_bind(), checkfirst=False)
