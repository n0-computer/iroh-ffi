//
//  iroh.h
//  iroh
//
//  Created by Brendan O'Brien on 5/15/23.
//

#import <Foundation/Foundation.h>

//! Project version number for iroh.
FOUNDATION_EXPORT double irohVersionNumber;

//! Project version string for iroh.
FOUNDATION_EXPORT const unsigned char irohVersionString[];

// In this header, you should import all the public headers of your framework using statements like #import <iroh/PublicHeader.h>

#ifndef __RUST_IROH_FFI__
#define __RUST_IROH_FFI__

#ifdef __cplusplus
extern "C" {
#endif


#include <stddef.h>
#include <stdint.h>

uint32_t iroh_get (
    char const * hash,
    char const * peer,
    char const * peer_addr,
    char const * out_path);

uint32_t iroh_get_ticket (
    char const * ticket,
    char const * out_path);


#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* __RUST_IROH_FFI__ */
