# Sanctioned Transfer Hook

The `sanction_transfer_hook` program provides an implementation of the transfer hook interface tht allows for block list functionality to disallow sets of addresses from being able to send/receive tokens.

Addresses included in the block list are evaluated against the source/recipient token account owners, as well as the address being used to sign the transfer. If any of the addresses are in the block list, the transfer is aborted.


# Architecture

## Management Account

The `Management` account is created through the `initialize` instruction which is intended to run immediately after the program is deployed. Creation of this account sets the authority to the address used to pay the rent cost.

Additional block lists can only be created by this authority.

## ExtraAccountMetaList Account

Standard account required by transfer hook implementations. This account must be created before any block lists are created.

## Block List Account

The main account of interest, which allows adding/removing addresses that can be blocked from sending/receiving tokens.

Whenever a new block list account is created, it is automatically added to the `ExtraAccountMetaList` account.


# "Gotchas"

## Front-Runnable Management Account Initialization

The initialization of the management account can be front-run immediately after program deployment. Possible solution is to hard code the seeds via anchor constraints to a specific address.


## Limited Block List Size

Due to `realloc` constraints, each block list can hold a max of 318 addresses. As a solution for this multiple block list accounts can be created.

## Fixed Max Address Size

At the moment the size of the block list account is assumed to have room for 318 addresses. A more convenient solution would be to allow customizing the amount of addresses that have space allocated for them, and then reallocate the account space when more addresses need to be added.