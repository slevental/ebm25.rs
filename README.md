# ebm25.rs
Searchable Symmetric Encryption with BM25 support 

## Disclaimer

There are multiple implementations of SEE, however there is no implementation that both doesn't leak information and efficient enough to be used in production. This implementation is no exception.

The main challenge is to prevent leakage of Access Pattern and Query Pattern. The former is the set of documents that were accessed by the user, the latter is the set of queries that were issued by the user. 

This repo is an attempt to implement SEE with BM25 support. It's not a production-ready implementation, it's a proof of concept.

## SEE

SEE is a cryptographic primitive that allows to search over encrypted data. It's a combination of symmetric encryption and order-preserving encryption.

## BM25

BM25 is a ranking function used by search engines to rank matching documents according to their relevance to a given search query. It's a bag-of-words model, i.e. it doesn't take into account the order of words in the document.

The main idea of BM25 (improved TF-IDF score) is to assign a score to each document based on the number of times the query terms appear in the document, while at the same time taking into account the length of the document. The score is computed as follows:

```BM25 = TF * IDF```

where:

```
IDF = log((N - n + 0.5) / (n + 0.5))
TF = f * (k1 + 1) / (f + k1 * (1 - b + b * DL / AVG_DL))
N - total number of documents in the collection
n - number of documents containing the term
f - frequency of the term in the document
DL - document length
AVG_DL - average document length in the collection
k1, b - free parameters, usually k1 = 1.2, b = 0.75
```

## References

- [Searchable Symmetric Encryption: Improved Definitions and Efficient Constructions](https://eprint.iacr.org/2006/210.pdf)
- [Secure Indexes](https://eprint.iacr.org/2003/216.pdf)
- [Multi-keyword Similarity Search Over Encrypted Cloud Data](https://eprint.iacr.org/2015/137.pdf)
- [SEAL: Attack Mitigation for Encrypted Databases via Adjustable Leakage](https://eprint.iacr.org/2019/811.pdf)
- [Order-preserving encryption (OPE)](https://github.com/sentclose/ope)
- [Order-Preserving Database Encryption with Secret Sharing](https://arxiv.org/pdf/2301.04370.pdf)
- [An Ideal-Security Protocol for Order-Preserving Encoding](https://people.csail.mit.edu/nickolai/papers/popa-mope-eprint.pdf)
- [Practical Techniques for Searches on Encrypted Data](https://people.eecs.berkeley.edu/~dawnsong/papers/se.pdf)
- [BUILDING PRACTICAL SYSTEMS THAT COMPUTE ON ENCRYPTED DATA](https://people.eecs.berkeley.edu/~raluca/Thesis.pdf)
- [CryptDB: Protecting Confidentiality with Encrypted Query Processing](http://people.csail.mit.edu/nickolai/papers/raluca-cryptdb.pdf)
- [Secure Search via Multi-Ring Fully Homomorphic Encryption](https://eprint.iacr.org/2018/245.pdf)
- [A Survey of Order-Preserving Encryption for Numeric Data](https://arxiv.org/pdf/1801.09933.pdf)