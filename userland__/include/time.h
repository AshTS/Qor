#ifndef _TIME_H
#define _TIME_H

struct rtc_time 
{
	int tm_sec;
	int tm_min;
	int tm_hour;
	int tm_mday;
	int tm_mon;
	int tm_year;
	int tm_wday;     /* unused */
	int tm_yday;     /* unused */
	int tm_isdst;    /* unused */
};

#endif _TIME_H