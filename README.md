# Encrypted BM25

[![Build Status](https://travis-ci.org/romankh3/encrypted-bm25.svg?branch=master)](https://travis-ci.org/romankh3/encrypted-bm25)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Searchable Symmetric Encryption with BM25 support

## Disclaimer

There are multiple implementations of SEE, however there is no implementation that both doesn't leak information and
efficient enough to be used in production. This implementation is no exception.

The main challenge is to prevent leakage of Access Pattern and Query Pattern. The former is the set of documents that
were accessed by the user, the latter is the set of queries that were issued by the user.

This repo is an attempt to implement SEE with BM25 support. It's not a production-ready implementation, it's a proof of
concept.

## SEE

Searchable Symmetric Encryption is allowing to perform search over encrypted data. The main idea is to encrypt the data
in such a way that it's possible to perform search over encrypted data without decrypting it. Unfortunately, it's not
possible to achieve this goal without leaking some information, but the goal is to leak as little information as
possible. Even with some leakage the algorith is still introducing significant overhead. This overhead is still much
less than one of the FHE or Functional encryption schemes.

| Operation                    | Memory                                        | Computation          |
|------------------------------|-----------------------------------------------|----------------------|
| Adding document to the index | O(document size + padding + terms * 64 bytes) | O(unique terms)      |
| Querying (worst case)        | O(document * terms * 64 bytes)                | O(terms * documents) |
| Querying (avg case)          | O(k * terms * 64 bytes)                       | O(k * terms)         |

Worst case scenario for the query is when all documents contain all terms. In this case the complexity is O(terms *
documents). But in the real world the number of documents that contain all terms is very small, so the complexity is
much less than O(terms * documents). And it could be limited on the client side.

In average case we can assume that the number of documents that term is present (k) is much less than the total number,
and could be limited since the IDF component of BM25 score would be very low for such terms.

## BM25

BM25 is a ranking function used by search engines to rank matching documents according to their relevance to a given
search query. It's a bag-of-words model, i.e. it doesn't take into account the order of words in the document.

The main idea of BM25 (improved TF-IDF score) is to assign a score to each document based on the number of times the
query terms appear in the document, while at the same time taking into account the length of the document. The score is
computed as follows:

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