from uuid import uuid4

from sqlmodel import Session, select

from app.exceptions import EntityAlreadyExists, EntityDoesNotExist
from app.models.translation import LanguageEnum, Translation
from app.schemas.translation import TranslationCreate, TranslationsCreate


def create_translations(
    session: Session, translations: TranslationsCreate
) -> tuple[str, list[Translation]]:
    """
    Create and store translation objects in the database.
    Args:
        session (Session): The database session to use for the operation.
        translations (list[NewTranslation]): A list of new translations to be added.
        key (str, optional): A unique key for the translations. Defaults to a new UUID.
    Returns:
        tuple[str, list[Translation]]: A tuple containing the key and the list of created translation objects.
    """
    # Generate a new key.
    key: str = translations.translation_key or str(uuid4())

    # Check if translations for the key already exist in the database.
    if translations.translation_key is not None:
        languages = [t.language for t in translations.translations]

        existing: list[LanguageEnum] = session.exec(
            select(Translation.language)
            .where(Translation.translation_key == key)
            .where(Translation.language.in_(languages))
        ).all()

        if existing:
            raise EntityAlreadyExists(
                f"Translations for key '{key}' already exist in languages: {', '.join([e.value for e in existing])}"
            )

    # Create translation objects.
    translations = [
        Translation(
            language=t.language,
            translation=t.translation,
            translation_key=key,
        )
        for t in translations.translations
    ]

    # Perform bulk insert.
    session.bulk_insert_mappings(Translation, translations)
    session.commit()

    # Return the key and the translation objects.
    return key, translations


def create_translation(
    session: Session, translation: TranslationCreate
) -> tuple[str, Translation]:
    """
    Create and store a translation object in the database.
    Args:
        session (Session): The database session to use for the operation.
        translation (NewTranslation): The new translation to be added.
        key (str, optional): A unique key for the translation. Defaults to None.
    Returns:
        tuple[str, Translation]: A tuple containing the key and the created translation object.
    """
    # If a key is not provided, generate a new one.
    key: str = translation.translation_key or str(uuid4())

    # Check if a translation for the key and language already exists in the database.
    if translation.translation_key is not None:
        existing = session.exec(
            select(Translation)
            .where(Translation.translation_key == key)
            .where(Translation.language == translation.language)
        ).first()

        if existing:
            raise EntityAlreadyExists(
                f"Translation for key '{key}' and language '{translation.language.value}' already exists"
            )

    # Create a translation object.
    translation_object = Translation(
        language=translation.language,
        translation_key=key,
        translation=translation.translation,
    )

    # Add the translation object to the session and commit.
    session.add(translation_object)
    session.commit()
    session.refresh(translation_object)

    # Return the translation object.
    return key, translation_object


def get_translations(session: Session, key: str) -> list[Translation]:
    """
    Get all translations with the given key.
    Args:
        session (Session): The database session to use for the operation.
        key (str): The key of the translations to be retrieved.
    Returns:
        list[Translation]: A list of translation objects with the given key.
    """
    # Get all translations with the given key.
    translations = session.exec(
        select(Translation).filter(Translation.translation_key == key)
    ).all()

    if len(translations) == 0:
        raise EntityDoesNotExist(f"Translations with key '{key}' not found")

    # Return the translations.
    return translations


def get_translation(session: Session, key: str, language: str) -> Translation:
    """
    Get the translation with the given key and language.
    Args:
        session (Session): The database session to use for the operation.
        key (str): The key of the translation to be retrieved.
        language (str): The language of the translation to be retrieved.
    Returns:
        Translation: The translation object with the given key and language.
    """
    # Get the translation with the given key and language.
    translation = (
        session.exec(Translation)
        .filter(
            Translation.translation_key == key, Translation.language == language
        )
        .first()
    )

    # Return the translation.
    return translation


def delete_translations(session: Session, key: str) -> None:
    """
    Delete all translations with the given key.
    Args:
        session (Session): The database session to use for the operation.
        key (str): The key of the translations to be deleted.
    """
    # Get all translations with the given key.
    translations = (
        session.exec(Translation)
        .filter(Translation.translation_key == key)
        .all()
    )

    # Delete the translations.
    for translation in translations:
        session.delete(translation)

    # Commit the transaction.
    session.commit()


def delete_translation(session: Session, key: str, language: str) -> None:
    """
    Delete the translation with the given key and language.
    Args:
        session (Session): The database session to use for the operation.
        key (str): The key of the translation to be deleted.
        language (str): The language of the translation to be deleted.
    """
    # Get the translation with the given key and language.
    translation = (
        session.exec(Translation)
        .filter(
            Translation.translation_key == key, Translation.language == language
        )
        .first()
    )

    # Delete the translation.
    session.delete(translation)

    # Commit the transaction.
    session.commit()
