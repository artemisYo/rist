#include <sys/ioctl.h>
#include <stdio.h>
#include <unistd.h>

int main (int argc, char **argv)
{
    printf("0x%x", TIOCGWINSZ);
    return 0;
}
