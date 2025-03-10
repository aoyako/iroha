"""
This module contains the AssetDefinition and Asset classes.
"""

from dataclasses import dataclass
from typing import Optional


@dataclass
class AssetDefinition:
    """
    AssetDefinition class represents an asset definition in the Iroha network.

    :param name: The name of the asset definition.
    :type name: str
    :param domain: The domain of the asset definition.
    :type domain: str
    :param scale: The numeric scale of the asset definition.
    :type scale: int
    """

    name: str
    domain: str
    scale: Optional[int] = None

    def __repr__(self):
        return f"{self.name}#{self.domain}"

    def get_id(self):
        """
        Get the asset definition ID.

        :return: The asset definition ID.
        :rtype: str
        """
        return f"{self.name}#{self.domain}"


@dataclass
class Asset:
    """
    Asset class represents an asset in the Iroha network.

    :param definition: The asset definition of the asset.
    :type definition: AssetDefinition
    :param account: The account of the asset.
    :type definition: str
    :param value: The value of the asset.
    :type value: str
    """

    definition: AssetDefinition
    account: str
    value: str

    def __repr__(self):
        return f"{self.definition.get_id()}:{self.value}"

    def get_value(self):
        """
        Get the value of the asset.

        :return: The value of the asset.
        :rtype: float
        """
        return self.value
