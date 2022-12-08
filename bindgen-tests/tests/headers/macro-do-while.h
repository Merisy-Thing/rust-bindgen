int xTaskDelayUntil(int, int);

#define vTaskDelayUntil( pxPreviousWakeTime, xTimeIncrement )                   \
    do {                                                                        \
        ( void ) xTaskDelayUntil( ( pxPreviousWakeTime ), ( xTimeIncrement ) ); \
    } while( 0 )
