// SPDX-License-Identifier: GPL-2.0-only

#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include "shuttlepro-report.h"

#define ARRAY_SIZE(a) (sizeof(a) / sizeof((a)[0]))

static int failures;

static void expect_bool(const char *name, bool actual, bool expected)
{
	if (actual == expected)
		return;

	fprintf(stderr, "%s: expected %s, got %s\n", name,
		expected ? "true" : "false", actual ? "true" : "false");
	failures++;
}

static void expect_int(const char *name, int actual, int expected)
{
	if (actual == expected)
		return;

	fprintf(stderr, "%s: expected %d, got %d\n", name, expected, actual);
	failures++;
}

static void expect_uint(const char *name, unsigned int actual,
			unsigned int expected)
{
	if (actual == expected)
		return;

	fprintf(stderr, "%s: expected 0x%x, got 0x%x\n", name, expected,
		actual);
	failures++;
}

static void test_rejects_invalid_inputs(void)
{
	unsigned char data[SHUTTLEPRO_REPORT_LEN] = { 0 };
	struct shuttlepro_report_state state = { 0 };
	struct shuttlepro_report report = { 0 };

	expect_bool("reject null data",
		    shuttlepro_decode_report(NULL, sizeof(data), &state,
					     &report),
		    false);
	expect_bool("reject null state",
		    shuttlepro_decode_report(data, sizeof(data), NULL,
					     &report),
		    false);
	expect_bool("reject null report",
		    shuttlepro_decode_report(data, sizeof(data), &state,
					     NULL),
		    false);
	expect_bool("reject short report",
		    shuttlepro_decode_report(data, SHUTTLEPRO_REPORT_LEN - 1,
					     &state, &report),
		    false);
}

static void test_shuttle_clamping(void)
{
	static const struct {
		unsigned char raw;
		int expected;
	} cases[] = {
		{ 0x00, 0 },
		{ 0x01, 1 },
		{ 0x07, 7 },
		{ 0x08, 7 },
		{ 0xff, -1 },
		{ 0xf9, -7 },
		{ 0xf8, -7 },
	};
	unsigned int i;

	for (i = 0; i < ARRAY_SIZE(cases); i++) {
		unsigned char data[SHUTTLEPRO_REPORT_LEN] = {
			cases[i].raw, 0, 0, 0, 0
		};
		struct shuttlepro_report_state state = { 0 };
		struct shuttlepro_report report = { 0 };

		expect_bool("decode clamp case",
			    shuttlepro_decode_report(data, sizeof(data), &state,
						     &report),
			    true);
		expect_int("shuttle clamp", report.shuttle, cases[i].expected);
	}
}

static void test_jog_baseline_and_delta(void)
{
	unsigned char first[SHUTTLEPRO_REPORT_LEN] = { 0, 10, 0, 0, 0 };
	unsigned char second[SHUTTLEPRO_REPORT_LEN] = { 0, 11, 0, 0, 0 };
	unsigned char third[SHUTTLEPRO_REPORT_LEN] = { 0, 9, 0, 0, 0 };
	struct shuttlepro_report_state state = { 0 };
	struct shuttlepro_report report = { 0 };

	expect_bool("decode first jog",
		    shuttlepro_decode_report(first, sizeof(first), &state,
					     &report),
		    true);
	expect_bool("first jog has no delta", report.has_jog_delta, false);
	expect_int("first jog delta zero", report.jog_delta, 0);

	expect_bool("decode positive jog",
		    shuttlepro_decode_report(second, sizeof(second), &state,
					     &report),
		    true);
	expect_bool("positive jog has delta", report.has_jog_delta, true);
	expect_int("positive jog delta", report.jog_delta, 1);

	expect_bool("decode negative jog",
		    shuttlepro_decode_report(third, sizeof(third), &state,
					     &report),
		    true);
	expect_bool("negative jog has delta", report.has_jog_delta, true);
	expect_int("negative jog delta", report.jog_delta, -2);
}

static void test_jog_wraparound(void)
{
	unsigned char first[SHUTTLEPRO_REPORT_LEN] = { 0, 255, 0, 0, 0 };
	unsigned char second[SHUTTLEPRO_REPORT_LEN] = { 0, 0, 0, 0, 0 };
	unsigned char third[SHUTTLEPRO_REPORT_LEN] = { 0, 255, 0, 0, 0 };
	struct shuttlepro_report_state state = { 0 };
	struct shuttlepro_report report = { 0 };

	expect_bool("decode wrap baseline",
		    shuttlepro_decode_report(first, sizeof(first), &state,
					     &report),
		    true);
	expect_bool("decode wrap forward",
		    shuttlepro_decode_report(second, sizeof(second), &state,
					     &report),
		    true);
	expect_int("wrap forward delta", report.jog_delta, 1);

	expect_bool("decode wrap backward",
		    shuttlepro_decode_report(third, sizeof(third), &state,
					     &report),
		    true);
	expect_int("wrap backward delta", report.jog_delta, -1);
}

static void test_button_masking(void)
{
	unsigned char data[SHUTTLEPRO_REPORT_LEN] = {
		0, 0, 0, 0xff, 0xff
	};
	struct shuttlepro_report_state state = { 0 };
	struct shuttlepro_report report = { 0 };

	expect_bool("decode buttons",
		    shuttlepro_decode_report(data, sizeof(data), &state,
					     &report),
		    true);
	expect_uint("button mask", report.buttons, SHUTTLEPRO_BUTTON_MASK);
}

int main(void)
{
	test_rejects_invalid_inputs();
	test_shuttle_clamping();
	test_jog_baseline_and_delta();
	test_jog_wraparound();
	test_button_masking();

	if (failures) {
		fprintf(stderr, "%d decode test failure(s)\n", failures);
		return 1;
	}

	return 0;
}
