class TradingCard:
    def __init__(self, title: str, description: str, rarity: str, isFoil: bool=False, isBorderless: bool=False, isFullart: bool=False):
        if isBorderless and isFullart:
            raise ValueError("A card cannot be both borderless and fullart at the same time.")
        
        self.title = title
        self.description = description
        self.rarity = rarity
        self.isFoil = isFoil
        self.isBorderless = isBorderless
        self.isFullart = isFullart

    def __str__(self):
        return (f"Title: {self.title}\n"
                f"Description: {self.description}\n"
                f"Rarity: {self.rarity}\n"
                f"Foil: {self.isFoil}\n"
                f"Borderless: {self.isBorderless}\n"
                f"Fullart: {self.isFullart}")