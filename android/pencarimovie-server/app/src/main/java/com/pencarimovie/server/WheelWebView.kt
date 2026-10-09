package com.pencarimovie.server

import android.content.Context
import android.util.AttributeSet
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.webkit.WebView

/**
 * WebView that translates mouse-wheel / trackpad scrolling (ACTION_SCROLL) into
 * page scrolling.
 *
 * Android's stock WebView — notably the Chrome 52 engine shipped on Android 5.x
 * that LDPlayer uses — ignores wheel events, so the web UI can only be scrolled
 * by touch/drag. We take the wheel delta and scroll whatever overflow container
 * sits under the pointer, because the Nuvio shell scrolls an inner element
 * (`.home-screen-main`) rather than the document (which is `overflow:hidden`).
 */
class WheelWebView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = 0
) : WebView(context, attrs, defStyleAttr) {

    override fun onGenericMotionEvent(event: MotionEvent): Boolean {
        if (event.action == MotionEvent.ACTION_SCROLL) {
            val v = event.getAxisValue(MotionEvent.AXIS_VSCROLL)
            val h = event.getAxisValue(MotionEvent.AXIS_HSCROLL)
            if (v != 0f || h != 0f) {
                val isMouse = event.isFromSource(InputDevice.SOURCE_MOUSE)
                // One wheel notch ≈ 120 px of content (browser default).
                val dy = (-v * 120f).toInt()
                val dx = (-h * 120f).toInt()
                scrollContentAt(event.x.toInt(), event.y.toInt(), dx, dy)
                // Consume true mouse/trackpad wheel input. For other sources
                // (some emulators map keys/remotes to scroll) we still scroll but
                // DON'T consume, so the default D-pad/keyboard handling is kept.
                return isMouse || super.onGenericMotionEvent(event)
            }
        }
        return super.onGenericMotionEvent(event)
    }

    /**
     * Emulators (and TV remotes) often send the mouse wheel / scroll as Up-Down
     * or Page keys. The stock WebView tries to scroll the document, but the shell
     * keeps `html`/`body` at `overflow:hidden` and scrolls an inner element, so
     * that does nothing. We scroll the real container ourselves.
     */
    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        val dy = when (keyCode) {
            KeyEvent.KEYCODE_DPAD_DOWN, KeyEvent.KEYCODE_PAGE_DOWN -> 180
            KeyEvent.KEYCODE_DPAD_UP, KeyEvent.KEYCODE_PAGE_UP -> -180
            else -> 0
        }
        if (dy != 0) {
            scrollContentAt(width / 2, height / 2, 0, dy)
            return true
        }
        return super.onKeyDown(keyCode, event)
    }

    private fun scrollContentAt(x: Int, y: Int, dx: Int, dy: Int) {
        val js = "(function(x,y,dx,dy){try{" +
            "function ok(n){return n&&n!==document&&n!==document.body&&n!==document.documentElement;}" +
            "function canV(n){var cs=getComputedStyle(n);return (cs.overflowY==='auto'||cs.overflowY==='scroll')&&n.scrollHeight>n.clientHeight+1;}" +
            "function canH(n){var cs=getComputedStyle(n);return (cs.overflowX==='auto'||cs.overflowX==='scroll')&&n.scrollWidth>n.clientWidth+1;}" +
            // Build the chain of scrollable ancestors from the pointer up, then
            // scroll the INNERMOST one first and chain outward at its boundary —
            // this is how a browser behaves, and it makes nested lists (e.g. the
            // stream list inside the metadata page) scroll instead of the page.
            "var chain=[],t=document.elementFromPoint(x,y);" +
            "while(t){if(ok(t)&&((dy!==0&&canV(t))||(dx!==0&&canH(t))))chain.push(t);t=t.parentElement;}" +
            "for(var i=0;i<chain.length;i++){var n=chain[i],b=n.scrollTop,bl=n.scrollLeft;" +
            "if(dy!==0&&canV(n)){n.scrollTop+=dy;if(n.scrollTop!==b)return 'ok';}" +
            "if(dx!==0&&canH(n)){n.scrollLeft+=dx;if(n.scrollLeft!==bl)return 'ok';}}" +
            "if(chain.length>0)return 'atEnd';" +
            // Fallback: the wheel event may carry no usable pointer position
            // (e.g. some remote/emulated mice report 0,0) → pick the largest
            // scrollable container instead.
            "var all=document.querySelectorAll('*'),best=null,bestH=-1;" +
            "for(var j=0;j<all.length;j++){var m=all[j];" +
            "if(ok(m)&&((dy!==0&&canV(m))||(dx!==0&&canH(m)))&&m.clientHeight>bestH&&m.getBoundingClientRect().height>0){best=m;bestH=m.clientHeight;}}" +
            "if(best){best.scrollTop+=dy;best.scrollLeft+=dx;return 'fallback';}" +
            "var d=document.scrollingElement||document.documentElement;" +
            "if(d&&d.scrollHeight>d.clientHeight+1){d.scrollTop+=dy;return 'doc';}" +
            "return 'none';}catch(e){return 'err';}})(" +
            x + "," + y + "," + dx + "," + dy + ");"
        evaluateJavascript(js, null)
    }
}
