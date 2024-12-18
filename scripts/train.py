import re

from tokenizers import Tokenizer, models, normalizers, pre_tokenizers, trainers
from datasets import load_dataset
from Stemmer import Stemmer
from nltk.corpus import stopwords


REGEX = re.compile(r"(?u)\b\w\w+\b")
stemmer = Stemmer("english")
stops = set(stopwords.words("english"))
ds = load_dataset("wikitext", "wikitext-103-raw-v1", split="train+test+validation")


def stemming(text, lowercase=True):
    words = REGEX.findall(text)
    return " ".join(
        word
        for word in stemmer.stemWords(
            word.lower() if lowercase else word for word in words
        )
        if word not in stops
    )


def batch_iterator(batch_size=1000):
    tok_dataset = ds.select_columns("text")
    for batch in tok_dataset.iter(batch_size):
        yield [stemming(text) for text in batch["text"]]


tk = Tokenizer(model=models.WordLevel(unk_token="[UNK]"))
# tk.normalizer = normalizers.Sequence(
#     [
#         normalizers.NFKC(),
#         normalizers.Lowercase(),
#         normalizers.StripAccents(),
#     ]
# )
tk.pre_tokenizer = pre_tokenizers.Whitespace()
trainer = trainers.WordLevelTrainer(vocab_size=500000, special_tokens=["[UNK]"])

tk.train_from_iterator(batch_iterator(), trainer=trainer, length=len(ds))
tk.save("wiki-103-word-level.json", pretty=False)
