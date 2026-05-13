/* SPDX-License-Identifier: GPL-2.0-only */
#ifndef SHUTTLEPRO_REPORT_H
#define SHUTTLEPRO_REPORT_H

#ifdef __KERNEL__
#include <linux/stddef.h>
#include <linux/types.h>
#else
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#endif

#define SHUTTLEPRO_REPORT_LEN		5
#define SHUTTLEPRO_BUTTONS		13
#define SHUTTLEPRO_BUTTON_MASK		((1U << SHUTTLEPRO_BUTTONS) - 1)
#define SHUTTLEPRO_SHUTTLE_MIN		(-7)
#define SHUTTLEPRO_SHUTTLE_MAX		7

struct shuttlepro_report_state {
	unsigned char jog;
	bool have_jog;
};

struct shuttlepro_report {
	int shuttle;
	unsigned int buttons;
	int jog_delta;
	bool has_jog_delta;
};

static inline int shuttlepro_s8(unsigned char value)
{
	return value & 0x80 ? (int)value - 0x100 : value;
}

static inline int shuttlepro_clamp_shuttle(int value)
{
	if (value < SHUTTLEPRO_SHUTTLE_MIN)
		return SHUTTLEPRO_SHUTTLE_MIN;
	if (value > SHUTTLEPRO_SHUTTLE_MAX)
		return SHUTTLEPRO_SHUTTLE_MAX;

	return value;
}

static inline bool shuttlepro_decode_report(const unsigned char *data,
					    size_t size,
					    struct shuttlepro_report_state *state,
					    struct shuttlepro_report *report)
{
	int delta;

	if (!data || !state || !report || size < SHUTTLEPRO_REPORT_LEN)
		return false;

	report->shuttle = shuttlepro_clamp_shuttle(shuttlepro_s8(data[0]));
	report->buttons = (data[3] | ((data[4] & 0x1f) << 8)) &
			  SHUTTLEPRO_BUTTON_MASK;
	report->jog_delta = 0;
	report->has_jog_delta = false;

	if (!state->have_jog) {
		state->jog = data[1];
		state->have_jog = true;
		return true;
	}

	delta = shuttlepro_s8(data[1] - state->jog);
	if (delta) {
		report->jog_delta = delta;
		report->has_jog_delta = true;
	}
	state->jog = data[1];

	return true;
}

#endif /* SHUTTLEPRO_REPORT_H */
