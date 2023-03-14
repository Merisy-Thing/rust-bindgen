void func(int, int);

#define Y 7
#define wrapper_func(x) func(x, Y)
#define func(x) func(x, Y)
