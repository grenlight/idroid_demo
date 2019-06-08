//
//  idroid.h
//  brush
//
//  Created by grenlight on 2019/5/28.
//  Copyright © 2019 grenlight. All rights reserved.
//

#ifndef idroid_h
#define idroid_h

#include <stdint.h>

#include "rs_glue.h"


// Create a new instance of `rust_obj`.
struct idroid_obj *create_triangle(struct app_view object);

void enter_frame(struct rust_obj *data);


#endif /* idroid_h */
